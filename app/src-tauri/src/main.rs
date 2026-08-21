#[cfg(target_os = "linux")]
fn configure_linux_webkit() {
    // Fedora 44 / WebKitGTK 2.52.5 can crash in EGL teardown
    // when accelerated compositing is enabled. This environment
    // variable is set before Tauri starts and before worker
    // threads are created so the WebKit child process inherits it.
    //
    // SAFETY: Rust 2024 marks process-environment mutation unsafe
    // because changing it concurrently with other threads can race.
    // This function runs at process startup before Tauri creates
    // threads, so there is no concurrent environment access here.
    unsafe {
        std::env::set_var(
            "WEBKIT_DISABLE_COMPOSITING_MODE",
            "1",
        );
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    configure_linux_webkit();

    al80_studio_app_lib::run()
}
