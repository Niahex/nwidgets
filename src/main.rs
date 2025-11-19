use gpui::{
    point, prelude::*, px, Application, Bounds, Size, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions,
};

use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

mod components;
mod ipc;
mod modules;
mod services;
mod theme;
mod widgets;

use widgets::osd;
use widgets::{AiChat, Panel, TranscriptionViewer};

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
            "chat" => {
                if let Err(e) = ipc::send_command(ipc::IpcCommand::ToggleAiChat) {
                    eprintln!("Failed to send command: {}", e);
                    eprintln!("Is nwidgets running?");
                    std::process::exit(1);
                }
                return;
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Available commands: dictation, chat");
                std::process::exit(1);
            }
        }
    }

    // Start main application
    Application::new().run(|cx| {
        // Initialize OSD event system
        use crate::services::OsdEventService;
        let osd_event_receiver = OsdEventService::init();

        // Initialize transcription event system
        use crate::services::TranscriptionEventService;
        let transcription_event_receiver = TranscriptionEventService::init();

        let panel_window = cx
            .open_window(
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
            |_, cx| {
                cx.new(|cx| osd::Osd::new(osd::OsdType::CapsLock(false), osd_event_receiver, cx))
            },
        )
        .unwrap();

        // Start IPC server for CLI commands
        let panel_entity = panel_window.update(cx, |_, _, cx| cx.entity()).unwrap();

        // Transcription window management
        let transcription_window: std::sync::Arc<
            std::sync::Mutex<Option<gpui::WindowHandle<TranscriptionViewer>>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(None));

        // Monitor transcription events and update/create viewer window
        let transcription_window_for_events = transcription_window.clone();
        let panel_for_stop = panel_entity.clone();
        cx.spawn(async move |cx| {
            use crate::services::receive_transcription_events;

            loop {
                gpui::Timer::after(std::time::Duration::from_millis(100)).await;

                if let Some(event) = receive_transcription_events(&transcription_event_receiver) {
                    match event {
                        crate::services::TranscriptionEvent::TextRecognized(text) => {
                            println!("[TRANSCRIPTION] Adding text to viewer: {}", text);

                            if let Some(window) =
                                transcription_window_for_events.lock().unwrap().as_ref()
                            {
                                let _ = window.update(cx, |viewer, _window, cx| {
                                    viewer.append_text(&text, cx);
                                });
                            }
                        }
                        crate::services::TranscriptionEvent::StopRequested => {
                            println!("[TRANSCRIPTION] Stop requested from UI");

                            // Stop recording in dictation module
                            let _ = panel_for_stop.update(cx, |panel, cx| {
                                if panel.dictation_module.is_recording() {
                                    panel.dictation_module.toggle_recording();

                                    use crate::services::{OsdEvent, OsdEventService};
                                    OsdEventService::send_event(OsdEvent::DictationStopped);

                                    cx.notify();
                                }
                            });

                            // Close window if still open
                            if let Some(window) =
                                transcription_window_for_events.lock().unwrap().take()
                            {
                                let _ = window.update(cx, |_, window, _| {
                                    window.remove_window();
                                });
                            }
                        }
                    }
                }
            }
        })
        .detach();

        let (ipc_tx, ipc_rx) = std::sync::mpsc::channel::<ipc::IpcCommand>();

        if let Err(e) = ipc::start_ipc_server(move |cmd| {
            let _ = ipc_tx.send(cmd);
        }) {
            eprintln!("[IPC] Failed to start IPC server: {}", e);
        }

        // AI Chat window management
        let ai_chat_window: std::sync::Arc<std::sync::Mutex<Option<gpui::WindowHandle<AiChat>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));

        // Poll for IPC commands
        let panel_for_ipc = panel_entity.clone();
        let transcription_window_for_ipc = transcription_window.clone();
        let ai_chat_window_for_ipc = ai_chat_window.clone();
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

                                    // Create transcription window
                                    println!("[IPC] Creating transcription window");
                                    match cx.open_window(
                                        WindowOptions {
                                            titlebar: None,
                                            window_bounds: Some(WindowBounds::Windowed(Bounds {
                                                origin: point(px(1420.), px(300.)),
                                                size: Size::new(px(600.), px(400.)),
                                            })),
                                            app_id: Some("nwidgets-transcription".to_string()),
                                            window_background:
                                                WindowBackgroundAppearance::Transparent,
                                            kind: WindowKind::LayerShell(LayerShellOptions {
                                                namespace: "nwidgets-transcription".to_string(),
                                                layer: Layer::Overlay,
                                                anchor: Anchor::empty(),
                                                keyboard_interactivity:
                                                    KeyboardInteractivity::Exclusive,
                                                ..Default::default()
                                            }),
                                            ..Default::default()
                                        },
                                        |_, cx| {
                                            cx.new(|_cx| TranscriptionViewer::new(String::new()))
                                        },
                                    ) {
                                        Ok(window) => {
                                            *transcription_window_for_ipc.lock().unwrap() =
                                                Some(window);
                                            println!("[IPC] Transcription window created");
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "[IPC] Failed to create transcription window: {:?}",
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    OsdEventService::send_event(OsdEvent::DictationStopped);

                                    // Close transcription window
                                    if let Some(window) =
                                        transcription_window_for_ipc.lock().unwrap().take()
                                    {
                                        println!("[IPC] Closing transcription window");
                                        let _ = window.update(cx, |_, window, _| {
                                            window.remove_window();
                                        });
                                    }
                                }

                                cx.notify();
                            });
                        }
                        ipc::IpcCommand::ToggleAiChat => {
                            println!("[IPC] Toggle AI chat command received");

                            // Check if chat window already exists
                            let mut window_lock = ai_chat_window_for_ipc.lock().unwrap();
                            if window_lock.is_some() {
                                // Close existing window
                                if let Some(window) = window_lock.take() {
                                    println!("[IPC] Closing AI chat window");
                                    let _ = window.update(cx, |_, window, _| {
                                        window.remove_window();
                                    });
                                }
                            } else {
                                // Create new chat window
                                println!("[IPC] Creating AI chat window");
                                match cx.open_window(
                                    WindowOptions {
                                        titlebar: None,
                                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                                            origin: point(px(0.), px(0.)),
                                            size: Size::new(px(400.), px(0.)), // Height ignored with TOP|BOTTOM anchor
                                        })),
                                        app_id: Some("nwidgets-ai-chat".to_string()),
                                        window_background: WindowBackgroundAppearance::Transparent,
                                        kind: WindowKind::LayerShell(LayerShellOptions {
                                            namespace: "nwidgets-ai-chat".to_string(),
                                            layer: Layer::Overlay,
                                            anchor: Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM,
                                            keyboard_interactivity:
                                                KeyboardInteractivity::Exclusive,
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    },
                                    |_, cx| cx.new(AiChat::new),
                                ) {
                                    Ok(window) => {
                                        *window_lock = Some(window);
                                        println!("[IPC] AI chat window created");
                                    }
                                    Err(e) => {
                                        eprintln!("[IPC] Failed to create AI chat window: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
        .detach();

        // Démarrer le service de notifications avec son manager
        use crate::services::notifications::NotificationManager;
        let _notification_manager = NotificationManager::new(cx);
        println!("[MAIN] 📢 Notification manager started");

        cx.activate(true);
    });
}
