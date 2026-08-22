use std::collections::VecDeque;

pub const EVENT_REPORT_BYTES: usize = 32;
pub const EVENT_NAMESPACE: u8 = 0x4C;
pub const EVENT_MARKER: u8 = 0xE1;
pub const EVENT_VERSION: u8 = 1;
pub const EVENT_QUEUE_CAPACITY: usize = 8;
pub const INPUT_BINDING_MAX: u8 = 12;
pub const INPUT_ACTION_MAX: u8 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InputEventKind {
    KnobCcw = 1,
    KnobCw = 2,
    KnobPress = 3,
}

impl TryFrom<u8> for InputEventKind {
    type Error = EventParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::KnobCcw),
            2 => Ok(Self::KnobCw),
            3 => Ok(Self::KnobPress),
            other => Err(EventParseError::InvalidEventKind(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TriggerKind {
    None = 0,
    Layer = 1,
    Matrix = 2,
    Mods = 3,
}

impl TryFrom<u8> for TriggerKind {
    type Error = EventParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Layer),
            2 => Ok(Self::Matrix),
            3 => Ok(Self::Mods),
            other => Err(EventParseError::InvalidTriggerKind(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputRouterEvent {
    pub sequence: u16,
    pub event: InputEventKind,
    pub slot: u8,
    pub trigger: TriggerKind,
    pub trigger_a: u8,
    pub trigger_b: u8,
    pub action: u8,
    pub dropped_counter: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventParseError {
    WrongVersion(u8),
    InvalidEventKind(u8),
    InvalidSlot(u8),
    InvalidTriggerKind(u8),
    InvalidAction(u8),
    NonZeroFlags(u8),
    RouterDisabled(u8),
    ReservedNonZero { offset: usize, value: u8 },
}

impl InputRouterEvent {
    pub fn parse(report: &[u8; EVENT_REPORT_BYTES]) -> Result<Self, EventParseError> {
        debug_assert_eq!(report.len(), EVENT_REPORT_BYTES);

        if report[2] != EVENT_VERSION {
            return Err(EventParseError::WrongVersion(report[2]));
        }

        let event = InputEventKind::try_from(report[5])?;

        let slot = report[6];

        if slot >= INPUT_BINDING_MAX {
            return Err(EventParseError::InvalidSlot(slot));
        }

        let trigger = TriggerKind::try_from(report[7])?;

        let action = report[10];

        if action > INPUT_ACTION_MAX {
            return Err(EventParseError::InvalidAction(action));
        }

        if report[11] != 0 {
            return Err(EventParseError::NonZeroFlags(report[11]));
        }

        if report[14] != 1 {
            return Err(EventParseError::RouterDisabled(report[14]));
        }

        for (offset, value) in report.iter().enumerate().skip(15) {
            if *value != 0 {
                return Err(EventParseError::ReservedNonZero {
                    offset,
                    value: *value,
                });
            }
        }

        Ok(Self {
            sequence: u16::from_le_bytes([report[3], report[4]]),
            event,
            slot,
            trigger,
            trigger_a: report[8],
            trigger_b: report[9],
            action,
            dropped_counter: u16::from_le_bytes([report[12], report[13]]),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceObservation {
    First(u16),
    Consecutive {
        previous: u16,
        current: u16,
    },
    Duplicate(u16),
    Gap {
        previous: u16,
        expected: u16,
        current: u16,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SequenceTracker {
    last: Option<u16>,
}

impl SequenceTracker {
    pub fn observe(&mut self, current: u16) -> SequenceObservation {
        let observation = match self.last {
            None => SequenceObservation::First(current),
            Some(previous) if current == previous => SequenceObservation::Duplicate(current),
            Some(previous) if current == previous.wrapping_add(1) => {
                SequenceObservation::Consecutive { previous, current }
            }
            Some(previous) => SequenceObservation::Gap {
                previous,
                expected: previous.wrapping_add(1),
                current,
            },
        };

        self.last = Some(current);
        observation
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportClass {
    RouterEvent(InputRouterEvent),
    MalformedRouterEvent(EventParseError),
    ResponseCandidate([u8; EVENT_REPORT_BYTES]),
}

pub fn classify_report(report: [u8; EVENT_REPORT_BYTES]) -> ReportClass {
    if report[0] == EVENT_NAMESPACE && report[1] == EVENT_MARKER {
        match InputRouterEvent::parse(&report) {
            Ok(event) => ReportClass::RouterEvent(event),
            Err(error) => ReportClass::MalformedRouterEvent(error),
        }
    } else {
        ReportClass::ResponseCandidate(report)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DemuxOutcome {
    EventQueued {
        event: InputRouterEvent,
        sequence: SequenceObservation,
    },
    EventDroppedHostQueueFull {
        event: InputRouterEvent,
        sequence: SequenceObservation,
    },
    MalformedEvent(EventParseError),
    MatchedResponse([u8; EVENT_REPORT_BYTES]),
    UnexpectedResponse([u8; EVENT_REPORT_BYTES]),
}

#[derive(Debug)]
pub struct EventDemux {
    events: VecDeque<InputRouterEvent>,
    sequence: SequenceTracker,
    malformed_events: u64,
    host_queue_drops: u64,
}

impl Default for EventDemux {
    fn default() -> Self {
        Self {
            events: VecDeque::with_capacity(EVENT_QUEUE_CAPACITY),
            sequence: SequenceTracker::default(),
            malformed_events: 0,
            host_queue_drops: 0,
        }
    }
}

impl EventDemux {
    pub fn accept(
        &mut self,
        report: [u8; EVENT_REPORT_BYTES],
        expected_response_namespace: Option<u8>,
    ) -> DemuxOutcome {
        match classify_report(report) {
            ReportClass::RouterEvent(event) => {
                let sequence = self.sequence.observe(event.sequence);

                if self.events.len() >= EVENT_QUEUE_CAPACITY {
                    self.host_queue_drops = self.host_queue_drops.saturating_add(1);

                    DemuxOutcome::EventDroppedHostQueueFull { event, sequence }
                } else {
                    self.events.push_back(event);

                    DemuxOutcome::EventQueued { event, sequence }
                }
            }
            ReportClass::MalformedRouterEvent(error) => {
                self.malformed_events = self.malformed_events.saturating_add(1);
                DemuxOutcome::MalformedEvent(error)
            }
            ReportClass::ResponseCandidate(response) => {
                if expected_response_namespace == Some(response[0]) {
                    DemuxOutcome::MatchedResponse(response)
                } else {
                    DemuxOutcome::UnexpectedResponse(response)
                }
            }
        }
    }

    pub fn pop_event(&mut self) -> Option<InputRouterEvent> {
        self.events.pop_front()
    }

    pub fn queued_events(&self) -> usize {
        self.events.len()
    }

    pub fn malformed_events(&self) -> u64 {
        self.malformed_events
    }

    pub fn host_queue_drops(&self) -> u64 {
        self.host_queue_drops
    }

    pub fn reset_for_disconnect(&mut self) {
        self.events.clear();
        self.sequence.reset();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingRequestError {
    Disconnected,
}

#[derive(Debug)]
pub struct PendingRequest {
    expected_namespace: u8,
    active: bool,
}

impl PendingRequest {
    pub fn new(expected_namespace: u8) -> Self {
        Self {
            expected_namespace,
            active: true,
        }
    }

    pub fn feed(
        &mut self,
        demux: &mut EventDemux,
        report: [u8; EVENT_REPORT_BYTES],
    ) -> Option<[u8; EVENT_REPORT_BYTES]> {
        if !self.active {
            return None;
        }

        match demux.accept(report, Some(self.expected_namespace)) {
            DemuxOutcome::MatchedResponse(response) => {
                self.active = false;
                Some(response)
            }
            DemuxOutcome::EventQueued { .. }
            | DemuxOutcome::EventDroppedHostQueueFull { .. }
            | DemuxOutcome::MalformedEvent(_)
            | DemuxOutcome::UnexpectedResponse(_) => None,
        }
    }

    pub fn disconnect(&mut self, demux: &mut EventDemux) -> Result<(), PendingRequestError> {
        demux.reset_for_disconnect();

        if self.active {
            self.active = false;
            Err(PendingRequestError::Disconnected)
        } else {
            Ok(())
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_report(
        sequence: u16,
        event: InputEventKind,
        slot: u8,
        trigger: TriggerKind,
        action: u8,
    ) -> [u8; EVENT_REPORT_BYTES] {
        let mut report = [0u8; EVENT_REPORT_BYTES];

        report[0] = EVENT_NAMESPACE;
        report[1] = EVENT_MARKER;
        report[2] = EVENT_VERSION;

        let sequence = sequence.to_le_bytes();
        report[3] = sequence[0];
        report[4] = sequence[1];

        report[5] = event as u8;
        report[6] = slot;
        report[7] = trigger as u8;
        report[8] = 0;
        report[9] = 0;
        report[10] = action;
        report[11] = 0;

        report[12] = 0;
        report[13] = 0;
        report[14] = 1;

        report
    }

    fn response(namespace: u8, status: u8) -> [u8; EVENT_REPORT_BYTES] {
        let mut report = [0u8; EVENT_REPORT_BYTES];
        report[0] = namespace;
        report[1] = status;
        report
    }

    #[test]
    fn response_only_matches_pending_request() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);
        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));
        assert!(!pending.is_active());
        assert_eq!(demux.queued_events(), 0);
    }

    #[test]
    fn event_only_is_queued_and_does_not_complete_request() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        let event = event_report(10, InputEventKind::KnobCw, 1, TriggerKind::None, 2);

        assert_eq!(pending.feed(&mut demux, event), None);
        assert!(pending.is_active());
        assert_eq!(demux.queued_events(), 1);
    }

    #[test]
    fn event_before_response_does_not_steal_response() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        assert_eq!(
            pending.feed(
                &mut demux,
                event_report(20, InputEventKind::KnobCcw, 0, TriggerKind::None, 1,),
            ),
            None
        );

        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));
        assert_eq!(demux.queued_events(), 1);
    }

    #[test]
    fn several_events_before_response_are_all_demultiplexed() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        for sequence in 30..34 {
            assert_eq!(
                pending.feed(
                    &mut demux,
                    event_report(sequence, InputEventKind::KnobCw, 2, TriggerKind::Layer, 23,),
                ),
                None
            );
        }

        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));
        assert_eq!(demux.queued_events(), 4);
    }

    #[test]
    fn response_then_event_is_still_classified_after_transaction() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);
        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));

        let outcome = demux.accept(
            event_report(40, InputEventKind::KnobPress, 3, TriggerKind::Matrix, 3),
            None,
        );

        assert!(matches!(outcome, DemuxOutcome::EventQueued { .. }));

        assert_eq!(demux.queued_events(), 1);
    }

    #[test]
    fn malformed_0x4c_event_is_rejected() {
        let mut demux = EventDemux::default();
        let mut report = event_report(50, InputEventKind::KnobCw, 0, TriggerKind::None, 2);

        report[10] = INPUT_ACTION_MAX + 1;

        assert!(matches!(
            demux.accept(report, None),
            DemuxOutcome::MalformedEvent(EventParseError::InvalidAction(25))
        ));

        assert_eq!(demux.malformed_events(), 1);
        assert_eq!(demux.queued_events(), 0);
    }

    #[test]
    fn sequence_wrap_65535_to_zero_is_consecutive() {
        let mut tracker = SequenceTracker::default();

        assert_eq!(
            tracker.observe(u16::MAX),
            SequenceObservation::First(u16::MAX)
        );

        assert_eq!(
            tracker.observe(0),
            SequenceObservation::Consecutive {
                previous: u16::MAX,
                current: 0,
            }
        );
    }

    #[test]
    fn sequence_gap_is_observed_but_not_fatal() {
        let mut demux = EventDemux::default();

        let first = demux.accept(
            event_report(100, InputEventKind::KnobCw, 0, TriggerKind::None, 2),
            None,
        );

        assert!(matches!(
            first,
            DemuxOutcome::EventQueued {
                sequence: SequenceObservation::First(100),
                ..
            }
        ));

        let gap = demux.accept(
            event_report(103, InputEventKind::KnobCw, 0, TriggerKind::None, 2),
            None,
        );

        assert!(matches!(
            gap,
            DemuxOutcome::EventQueued {
                sequence: SequenceObservation::Gap {
                    previous: 100,
                    expected: 101,
                    current: 103,
                },
                ..
            }
        ));

        assert_eq!(demux.queued_events(), 2);
    }

    #[test]
    fn event_queue_overflow_does_not_block_response() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        for sequence in 0..EVENT_QUEUE_CAPACITY as u16 {
            assert!(matches!(
                demux.accept(
                    event_report(sequence, InputEventKind::KnobCw, 0, TriggerKind::None, 2,),
                    Some(0x4B),
                ),
                DemuxOutcome::EventQueued { .. }
            ));
        }

        assert_eq!(demux.queued_events(), EVENT_QUEUE_CAPACITY);

        assert!(matches!(
            demux.accept(
                event_report(
                    EVENT_QUEUE_CAPACITY as u16,
                    InputEventKind::KnobCw,
                    0,
                    TriggerKind::None,
                    2,
                ),
                Some(0x4B),
            ),
            DemuxOutcome::EventDroppedHostQueueFull { .. }
        ));

        assert_eq!(demux.host_queue_drops(), 1);

        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));
    }

    #[test]
    fn disconnect_fails_pending_request_and_resets_transient_state() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        assert_eq!(
            pending.feed(
                &mut demux,
                event_report(200, InputEventKind::KnobPress, 0, TriggerKind::None, 3,),
            ),
            None
        );

        assert_eq!(demux.queued_events(), 1);

        assert_eq!(
            pending.disconnect(&mut demux),
            Err(PendingRequestError::Disconnected)
        );

        assert!(!pending.is_active());
        assert_eq!(demux.queued_events(), 0);

        let outcome = demux.accept(
            event_report(0, InputEventKind::KnobCw, 0, TriggerKind::None, 2),
            None,
        );

        assert!(matches!(
            outcome,
            DemuxOutcome::EventQueued {
                sequence: SequenceObservation::First(0),
                ..
            }
        ));
    }

    #[test]
    fn wrong_response_namespace_does_not_complete_pending_request() {
        let mut demux = EventDemux::default();
        let mut pending = PendingRequest::new(0x4B);

        assert_eq!(pending.feed(&mut demux, response(0x49, 0x55),), None);

        assert!(pending.is_active());

        let expected = response(0x4B, 0x55);

        assert_eq!(pending.feed(&mut demux, expected), Some(expected));
    }

    #[test]
    fn reserved_bytes_must_be_zero() {
        let mut report = event_report(300, InputEventKind::KnobCw, 0, TriggerKind::None, 2);

        report[31] = 1;

        assert_eq!(
            InputRouterEvent::parse(&report),
            Err(EventParseError::ReservedNonZero {
                offset: 31,
                value: 1,
            })
        );
    }

    #[test]
    fn router_enabled_byte_must_be_one() {
        let mut report = event_report(301, InputEventKind::KnobCw, 0, TriggerKind::None, 2);

        report[14] = 0;

        assert_eq!(
            InputRouterEvent::parse(&report),
            Err(EventParseError::RouterDisabled(0))
        );
    }
}
