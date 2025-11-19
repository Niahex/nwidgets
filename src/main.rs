use gpui::{
    point, prelude::*, px, Application, Bounds, Size,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};

use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

mod ipc;
mod modules;
mod services;
mod theme;
mod widgets;

use widgets::osd;
use widgets::Panel;

fn main() {
    // Check if this is a CLI command
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "dictation" => {
                if let Err(e) = ipc::send_command(ipc::IpcCommand::ToggleDictation) {
                    eprintln!("Failed to send command: {}", e);
                    eprintln!("Is nwidgets running?");
                    std::process::exit(1);
                }
                return;
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Available commands: dictation");
                std::process::exit(1);
            }
        }
    }

    // Start main application
    Application::new().run(|cx| {
        // Initialize OSD event system
        use crate::services::OsdEventService;
        let osd_event_receiver = OsdEventService::init();

        let panel_window = cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(3440.), px(48.)),
                })),
                app_id: Some("nwidgets-panel".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-panel".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(px(48.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Panel::new),
        )
        .unwrap();

        // Start IPC server for CLI commands
        let panel_entity = panel_window.update(cx, |_, _, cx| cx.entity()).unwrap();

        let (ipc_tx, ipc_rx) = std::sync::mpsc::channel::<ipc::IpcCommand>();

        if let Err(e) = ipc::start_ipc_server(move |cmd| {
            let _ = ipc_tx.send(cmd);
        }) {
            eprintln!("[IPC] Failed to start IPC server: {}", e);
        }

        // Poll for IPC commands
        let panel_for_ipc = panel_entity.clone();
        let window_for_ipc = panel_window.clone();
        cx.spawn(async move |cx| {
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(100)).await;

                if let Ok(cmd) = ipc_rx.try_recv() {
                    match cmd {
                        ipc::IpcCommand::ToggleDictation => {
                            println!("[IPC] Toggle dictation command received");

                            use crate::services::{OsdEvent, OsdEventService};

                            let _ = panel_for_ipc.update(cx, |panel, cx| {
                                panel.dictation_module.toggle_recording();

                                let is_recording = panel.dictation_module.is_recording();
                                if is_recording {
                                    OsdEventService::send_event(OsdEvent::DictationStarted);
                                } else {
                                    OsdEventService::send_event(OsdEvent::DictationStopped);
                                }

                                cx.notify();
                            });
                        }
                    }
                }
            }
        })
        .detach();

        // OSD - centered at bottom with margin
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(1520.), px(920.)), // Center horizontally (3440/2 - 400/2 = 1520), 60px margin from bottom
                    size: Size::new(px(400.), px(64.)),
                })),
                app_id: Some("nwidgets-osd".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-osd".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::BOTTOM,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    margin: Some((px(0.), px(0.), px(60.), px(0.))), // top, right, bottom, left
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| osd::Osd::new(osd::OsdType::CapsLock(false), osd_event_receiver, cx)),
        )
        .unwrap();

        // Démarrer le service de notifications avec son manager
        use crate::services::notifications::NotificationManager;
        let _notification_manager = NotificationManager::new(cx);
        println!("[MAIN] 📢 Notification manager started");

        cx.activate(true);
    });
}
