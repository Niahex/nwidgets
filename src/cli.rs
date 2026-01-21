pub fn init_cef() {
    if let Err(e) = crate::services::cef::initialize_cef() {
        log::error!("Failed to initialize CEF: {e:?}");
        std::process::exit(1);
    }
    log::info!("CEF initialized successfully");
}

fn send_dbus_command(method: &str) -> bool {
    std::process::Command::new("dbus-send")
        .args([
            "--session",
            "--type=method_call",
            "--dest=org.nwidgets.App",
            "/org/nwidgets/App",
            &format!("org.nwidgets.App.{method}"),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn handle_command(command: &str) {
    match command {
        "chat" => {
            if send_dbus_command("ToggleChat") {
                std::process::exit(0);
            } else {
                log::error!("nwidgets is not running or D-Bus call failed");
                std::process::exit(1);
            }
        }
        _ => {
            log::error!("Unknown command: {command}");
            log::info!("Usage: nwidgets [chat]");
            std::process::exit(1);
        }
    }
}
