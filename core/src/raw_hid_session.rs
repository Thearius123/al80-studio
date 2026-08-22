use std::collections::VecDeque;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::input_event_bridge::{
    classify_report, InputRouterEvent, ReportClass, SequenceObservation, SequenceTracker,
    EVENT_QUEUE_CAPACITY, EVENT_REPORT_BYTES,
};

const REPORT_WITH_ID_BYTES: usize = EVENT_REPORT_BYTES + 1;
const REPORT_ID: u8 = 0;
const IDLE_POLL: Duration = Duration::from_millis(1);
const WRITE_RETRY_SLEEP: Duration = Duration::from_millis(1);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);
const CALLER_GRACE: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RawHidSessionStats {
    pub events_received: u64,
    pub malformed_events: u64,
    pub host_event_queue_drops: u64,
    pub unexpected_responses: u64,
    pub sequence_gaps: u64,
    pub sequence_duplicates: u64,
}

#[derive(Default)]
struct AtomicStats {
    events_received: AtomicU64,
    malformed_events: AtomicU64,
    host_event_queue_drops: AtomicU64,
    unexpected_responses: AtomicU64,
    sequence_gaps: AtomicU64,
    sequence_duplicates: AtomicU64,
}

impl AtomicStats {
    fn snapshot(&self) -> RawHidSessionStats {
        RawHidSessionStats {
            events_received: self.events_received.load(Ordering::Relaxed),
            malformed_events: self.malformed_events.load(Ordering::Relaxed),
            host_event_queue_drops: self.host_event_queue_drops.load(Ordering::Relaxed),
            unexpected_responses: self.unexpected_responses.load(Ordering::Relaxed),
            sequence_gaps: self.sequence_gaps.load(Ordering::Relaxed),
            sequence_duplicates: self.sequence_duplicates.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostInputEvent {
    pub event: InputRouterEvent,
    pub sequence: SequenceObservation,
}

type TransactionResult = Result<([u8; EVENT_REPORT_BYTES], f64), String>;

struct TransactionRequest {
    payload: [u8; EVENT_REPORT_BYTES],
    timeout: Duration,
    reply: SyncSender<TransactionResult>,
}

enum WorkerCommand {
    Transaction(TransactionRequest),
    Shutdown,
}

struct PendingTransaction {
    expected_namespace: u8,
    started: Instant,
    deadline: Instant,
    reply: SyncSender<TransactionResult>,
}

pub struct RawHidSession {
    command_tx: Sender<WorkerCommand>,
    event_queue: Arc<Mutex<VecDeque<HostInputEvent>>>,
    stats: Arc<AtomicStats>,
    last_error: Arc<Mutex<Option<String>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl RawHidSession {
    pub fn new(file: File) -> Result<Self, String> {
        let (command_tx, command_rx) = mpsc::channel();

        let event_queue = Arc::new(Mutex::new(VecDeque::with_capacity(EVENT_QUEUE_CAPACITY)));

        let stats = Arc::new(AtomicStats::default());
        let last_error = Arc::new(Mutex::new(None));

        let worker_event_queue = Arc::clone(&event_queue);
        let worker_stats = Arc::clone(&stats);
        let worker_last_error = Arc::clone(&last_error);

        let worker = thread::Builder::new()
            .name("al80-raw-hid-reader".to_string())
            .spawn(move || {
                worker_main(
                    file,
                    command_rx,
                    worker_event_queue,
                    worker_stats,
                    worker_last_error,
                )
            })
            .map_err(|error| format!("Raw HID I/O worker spawn failed: {error}"))?;

        Ok(Self {
            command_tx,
            event_queue,
            stats,
            last_error,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub fn transact(
        &self,
        payload: &[u8; EVENT_REPORT_BYTES],
        timeout: Duration,
    ) -> TransactionResult {
        if let Some(error) = self.last_error()? {
            return Err(error);
        }

        let (reply_tx, reply_rx) = mpsc::sync_channel(1);

        self.command_tx
            .send(WorkerCommand::Transaction(TransactionRequest {
                payload: *payload,
                timeout,
                reply: reply_tx,
            }))
            .map_err(|_| self.worker_error("Raw HID I/O worker command channel closed"))?;

        match reply_rx.recv_timeout(WRITE_TIMEOUT + timeout + CALLER_GRACE) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                Err(self.worker_error("Raw HID I/O worker response timeout"))
            }
            Err(RecvTimeoutError::Disconnected) => {
                Err(self.worker_error("Raw HID I/O worker response channel closed"))
            }
        }
    }

    pub fn pop_input_event(&self) -> Result<Option<HostInputEvent>, String> {
        let event = {
            let mut queue = self
                .event_queue
                .lock()
                .map_err(|_| "Raw HID event queue lock poisoned".to_string())?;

            queue.pop_front()
        };

        if event.is_some() {
            return Ok(event);
        }

        if let Some(error) = self.last_error()? {
            return Err(error);
        }

        Ok(None)
    }

    pub fn queued_input_events(&self) -> Result<usize, String> {
        let queue = self
            .event_queue
            .lock()
            .map_err(|_| "Raw HID event queue lock poisoned".to_string())?;

        Ok(queue.len())
    }

    pub fn stats(&self) -> RawHidSessionStats {
        self.stats.snapshot()
    }

    pub fn last_error(&self) -> Result<Option<String>, String> {
        let error = self
            .last_error
            .lock()
            .map_err(|_| "Raw HID worker error lock poisoned".to_string())?;

        Ok(error.clone())
    }

    fn worker_error(&self, fallback: &str) -> String {
        match self.last_error() {
            Ok(Some(error)) => error,
            Ok(None) | Err(_) => fallback.to_string(),
        }
    }
}

impl Drop for RawHidSession {
    fn drop(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);

        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}

fn worker_main(
    mut file: File,
    command_rx: Receiver<WorkerCommand>,
    event_queue: Arc<Mutex<VecDeque<HostInputEvent>>>,
    stats: Arc<AtomicStats>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let mut pending: Option<PendingTransaction> = None;
    let mut sequence = SequenceTracker::default();

    loop {
        let mut progressed = false;

        if pending.is_none() {
            match command_rx.try_recv() {
                Ok(WorkerCommand::Transaction(request)) => {
                    progressed = true;
                    let started = Instant::now();

                    match write_report(&mut file, &request.payload, WRITE_TIMEOUT) {
                        Ok(()) => {
                            pending = Some(PendingTransaction {
                                expected_namespace: request.payload[0],
                                started,
                                deadline: started + request.timeout,
                                reply: request.reply,
                            });
                        }
                        Err(error) => {
                            let _ = request.reply.send(Err(error.clone()));
                            set_worker_error(&last_error, error);
                            break;
                        }
                    }
                }
                Ok(WorkerCommand::Shutdown) => break,
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => break,
            }
        }

        match read_one_report(&mut file) {
            Ok(Some(report)) => {
                progressed = true;

                match classify_report(report) {
                    ReportClass::RouterEvent(event) => {
                        stats.events_received.fetch_add(1, Ordering::Relaxed);

                        let observation = sequence.observe(event.sequence);

                        match observation {
                            SequenceObservation::Gap { .. } => {
                                stats.sequence_gaps.fetch_add(1, Ordering::Relaxed);
                            }
                            SequenceObservation::Duplicate(_) => {
                                stats.sequence_duplicates.fetch_add(1, Ordering::Relaxed);
                            }
                            SequenceObservation::First(_)
                            | SequenceObservation::Consecutive { .. } => {}
                        }

                        queue_event(
                            &event_queue,
                            &stats,
                            HostInputEvent {
                                event,
                                sequence: observation,
                            },
                        );
                    }
                    ReportClass::MalformedRouterEvent(_) => {
                        stats.malformed_events.fetch_add(1, Ordering::Relaxed);
                    }
                    ReportClass::ResponseCandidate(response) => {
                        let matches_pending = pending
                            .as_ref()
                            .map(|request| response[0] == request.expected_namespace)
                            .unwrap_or(false);

                        if matches_pending {
                            if let Some(request) = pending.take() {
                                let elapsed_ms = request.started.elapsed().as_secs_f64() * 1000.0;

                                let _ = request.reply.send(Ok((response, elapsed_ms)));
                            }
                        } else {
                            stats.unexpected_responses.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                if let Some(request) = pending.take() {
                    let _ = request.reply.send(Err(error.clone()));
                }

                set_worker_error(&last_error, error);
                break;
            }
        }

        if let Some(request) = pending.as_ref() {
            if Instant::now() >= request.deadline {
                if let Some(request) = pending.take() {
                    let elapsed_ms = request.started.elapsed().as_secs_f64() * 1000.0;

                    let _ = request.reply.send(Err(format!(
                        "Raw HID response timeout after {elapsed_ms:.3} ms"
                    )));
                }
            }
        }

        if pending.is_none() {
            match command_rx.try_recv() {
                Ok(WorkerCommand::Transaction(request)) => {
                    progressed = true;
                    let started = Instant::now();

                    match write_report(&mut file, &request.payload, WRITE_TIMEOUT) {
                        Ok(()) => {
                            pending = Some(PendingTransaction {
                                expected_namespace: request.payload[0],
                                started,
                                deadline: started + request.timeout,
                                reply: request.reply,
                            });
                        }
                        Err(error) => {
                            let _ = request.reply.send(Err(error.clone()));
                            set_worker_error(&last_error, error);
                            break;
                        }
                    }
                }
                Ok(WorkerCommand::Shutdown) => break,
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => break,
            }
        }

        if !progressed {
            thread::sleep(IDLE_POLL);
        }
    }

    if let Some(request) = pending.take() {
        let _ = request.reply.send(Err(
            "Raw HID I/O worker stopped with request pending".to_string()
        ));
    }
}

fn queue_event(
    queue: &Arc<Mutex<VecDeque<HostInputEvent>>>,
    stats: &AtomicStats,
    event: HostInputEvent,
) {
    match queue.lock() {
        Ok(mut queue) => {
            if queue.len() >= EVENT_QUEUE_CAPACITY {
                stats.host_event_queue_drops.fetch_add(1, Ordering::Relaxed);
            } else {
                queue.push_back(event);
            }
        }
        Err(_) => {
            stats.host_event_queue_drops.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn write_report(
    file: &mut File,
    payload: &[u8; EVENT_REPORT_BYTES],
    timeout: Duration,
) -> Result<(), String> {
    let started = Instant::now();

    let mut report = [0u8; REPORT_WITH_ID_BYTES];
    report[0] = REPORT_ID;
    report[1..].copy_from_slice(payload);

    let mut written = 0usize;

    while written < report.len() {
        match file.write(&report[written..]) {
            Ok(0) => {
                return Err("Raw HID I/O write returned zero bytes".to_string());
            }
            Ok(count) => {
                written += count;
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if started.elapsed() >= timeout {
                    return Err("Raw HID I/O write timeout".to_string());
                }

                thread::sleep(WRITE_RETRY_SLEEP);
            }
            Err(error) => {
                return Err(format!("Raw HID I/O write failed: {error}"));
            }
        }
    }

    Ok(())
}

fn read_one_report(file: &mut File) -> Result<Option<[u8; EVENT_REPORT_BYTES]>, String> {
    let mut buffer = [0u8; REPORT_WITH_ID_BYTES];

    match file.read(&mut buffer) {
        Ok(0) => Ok(None),

        Ok(EVENT_REPORT_BYTES) => {
            let mut report = [0u8; EVENT_REPORT_BYTES];
            report.copy_from_slice(&buffer[..EVENT_REPORT_BYTES]);
            Ok(Some(report))
        }

        Ok(REPORT_WITH_ID_BYTES) if buffer[0] == REPORT_ID => {
            let mut report = [0u8; EVENT_REPORT_BYTES];
            report.copy_from_slice(&buffer[1..]);
            Ok(Some(report))
        }

        Ok(REPORT_WITH_ID_BYTES) => Err(format!(
            "Raw HID I/O read returned unexpected report ID: {}",
            buffer[0]
        )),

        Ok(count) => Err(format!(
            "Raw HID I/O read returned unexpected report size: {count}"
        )),

        Err(error) if error.kind() == ErrorKind::Interrupted => Ok(None),
        Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(None),

        Err(error) => Err(format!("Raw HID I/O read failed: {error}")),
    }
}

fn set_worker_error(last_error: &Arc<Mutex<Option<String>>>, error: String) {
    if let Ok(mut slot) = last_error.lock() {
        *slot = Some(error);
    }
}
