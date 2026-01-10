mod components;
mod services;
mod theme;
mod utils;
mod widgets;

use anyhow::Result;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::prelude::*;
use gpui::*;
use gpui::{Bounds, Point, Size, WindowBounds};
use parking_lot::Mutex;
use services::{
    audio::AudioService,
    bluetooth::BluetoothService,
    cef::CefService,
    chat::{ChatService, ChatToggled},
    control_center::ControlCenterService,
    hyprland::HyprlandService,
    mpris::MprisService,
    network::NetworkService,
    notifications::{NotificationAdded, NotificationService},
    osd::OsdService,
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use widgets::{
    chat::ChatWidget,
    notifications::{NotificationsStateChanged, NotificationsWindowManager},
    panel::Panel,
};

struct Assets {
    base: PathBuf,
    cache: parking_lot::RwLock<HashMap<String, &'static [u8]>>,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        {
            let cache = self.cache.read();
            if let Some(data) = cache.get(path) {
                return Ok(Some(std::borrow::Cow::Borrowed(data)));
            }
        }

        match std::fs::read(self.base.join(path)) {
            Ok(data) => {
                let leaked_data: &'static [u8] = Box::leak(data.into_boxed_slice());
                let mut cache = self.cache.write();
                cache.insert(path.to_string(), leaked_data);
                Ok(Some(std::borrow::Cow::Borrowed(leaked_data)))
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        std::fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

fn init_cef() {
    if let Err(e) = services::cef::initialize_cef() {
        eprintln!("Failed to initialize CEF: {:?}", e);
        std::process::exit(1);
    }
    eprintln!("CEF initialized successfully!");
}

const SOCKET_PATH: &str = "/tmp/nwidgets.sock";

fn send_command(cmd: &str) -> bool {
    match UnixStream::connect(SOCKET_PATH) {
        Ok(mut stream) => {
            let _ = stream.write_all(cmd.as_bytes());
            true
        }
        Err(e) => {
            eprintln!("Failed to connect to socket: {}", e);
            false
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // CEF subprocess - let CEF handle it
    if args.iter().any(|a| a.starts_with("--type=")) {
        init_cef();
        return;
    }

    // CLI command: nwidgets chat
    if args.len() > 1 {
        match args[1].as_str() {
            "chat" => {
                if send_command("toggle_chat") {
                    std::process::exit(0);
                } else {
                    eprintln!("nwidgets is not running. Start it first with: nwidgets");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Usage: nwidgets [chat]");
                std::process::exit(1);
            }
        }
    }

    // No args - start the full app
    init_cef();

    // Determine assets path
    let assets_path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    };

    Application::new()
        .with_assets(Assets {
            base: assets_path,
            cache: parking_lot::RwLock::new(HashMap::new()),
        })
        .run(|cx: &mut App| {
            gpui_tokio::init(cx);
            cx.set_global(theme::Theme::nord_dark());

            // Initialize global services
            HyprlandService::init(cx);
            AudioService::init(cx);
            BluetoothService::init(cx);
            NetworkService::init(cx);
            MprisService::init(cx);
            PomodoroService::init(cx);
            SystrayService::init(cx);
            CefService::init(cx);
            let chat_service = ChatService::init(cx);
            let notif_service = NotificationService::init(cx);
            let osd_service = OsdService::init(cx);
            ControlCenterService::init(cx);

            // Initialize CEF Service
            services::cef::CefService::init(cx);

            // Gestionnaire de fenêtre notifications
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(0.0), y: px(0.0) },
                        size: Size { width: px(3440.0), height: px(50.0) },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-panel".to_string(),
                        layer: Layer::Top,
                        anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                        exclusive_zone: Some(px(50.)),
                        margin: None,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(Panel::new),
            ).unwrap();

            // Chat window state - open at startup
            // Chat window - NOT opened at startup
            let chat_window: Arc<Mutex<Option<WindowHandle<ChatWidget>>>> = Arc::new(Mutex::new(None));
            let chat_window_clone = Arc::clone(&chat_window);
            let chat_window_clone2 = Arc::clone(&chat_window);

            // Subscribe to chat toggle events
            cx.subscribe(&chat_service, move |_service, _event: &ChatToggled, cx| {
                let mut window = chat_window_clone.lock();
                if let Some(handle) = window.take() {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                } else {
                    let handle = cx.open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(Bounds {
                                origin: Point { x: px(0.0), y: px(0.0) },
                                size: Size { width: px(600.0), height: px(1370.0) },
                            })),
                            titlebar: None,
                            window_background: WindowBackgroundAppearance::Transparent,
                            kind: WindowKind::LayerShell(LayerShellOptions {
                                namespace: "nwidgets-chat".to_string(),
                                layer: Layer::Overlay,
                                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
                                exclusive_zone: None,
                                margin: Some((px(50.0), px(0.0), px(0.0), px(0.0))),
                                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                                ..Default::default()
                            }),
                            app_id: Some("nwidgets-chat".to_string()),
                            ..Default::default()
                        },
                        |_window, cx| cx.new(ChatWidget::new),
                    ).ok();
                    *window = handle;
                }
            }).detach();

            // Socket server for IPC
            let _ = std::fs::remove_file(SOCKET_PATH);
            if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {
                listener.set_nonblocking(true).ok();

                cx.spawn(|cx: &mut AsyncApp| {
                    let chat_window = chat_window_clone2;
                    let mut cx = cx.clone();
                    async move {
                        loop {
                            cx.background_executor().timer(std::time::Duration::from_millis(100)).await;

                            if let Ok((mut stream, _)) = listener.accept() {
                                let mut buf = [0u8; 64];
                                if let Ok(n) = stream.read(&mut buf) {
                                    let cmd = String::from_utf8_lossy(&buf[..n]);
                                    if cmd.trim() == "toggle_chat" {
                                        let _ = cx.update(|cx: &mut App| {
                                            let mut window = chat_window.lock();
                                            if let Some(handle) = window.take() {
                                                let _ = handle.update(cx, |_, window, _| window.remove_window());
                                            } else {
                                                let handle = cx.open_window(
                                                    WindowOptions {
                                                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                                                            origin: Point { x: px(0.0), y: px(0.0) },
                                                            size: Size { width: px(600.0), height: px(1370.0) },
                                                        })),
                                                        titlebar: None,
                                                        window_background: WindowBackgroundAppearance::Transparent,
                                                        kind: WindowKind::LayerShell(LayerShellOptions {
                                                            namespace: "nwidgets-chat".to_string(),
                                                            layer: Layer::Overlay,
                                                            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
                                                            exclusive_zone: None,
                                                            margin: Some((px(50.0), px(0.0), px(0.0), px(0.0))),
                                                            keyboard_interactivity: KeyboardInteractivity::OnDemand,
                                                            ..Default::default()
                                                        }),
                                                        app_id: Some("nwidgets-chat".to_string()),
                                                        ..Default::default()
                                                    },
                                                    |_window, cx| cx.new(ChatWidget::new),
                                                ).ok();
                                                *window = handle;
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }).detach();
            }

            let _osd_service = osd_service;
            let notif_manager = Arc::new(Mutex::new(NotificationsWindowManager::new()));
            let notif_manager_clone = Arc::clone(&notif_manager);

            cx.subscribe(
                &notif_service,
                move |_service, _event: &NotificationAdded, cx| {
                    let mut manager = notif_manager_clone.lock();
                    if let Some(widget) = manager.open_window(cx) {
                        let notif_manager_clone2 = Arc::clone(&notif_manager_clone);
                        cx.subscribe(
                            &widget,
                            move |_widget, event: &NotificationsStateChanged, cx| {
                                if !event.has_notifications {
                                    let mut manager = notif_manager_clone2.lock();
                                    manager.close_window(cx);
                                }
                            },
                        ).detach();
                    }
                },
            ).detach();

            cx.activate(true);
        });
}
