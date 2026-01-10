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
    chat::{ChatNavigate, ChatPinToggled, ChatService, ChatToggled},
    control_center::ControlCenterService,
    dbus::DbusService,
    hyprland::HyprlandService,
    mpris::MprisService,
    network::NetworkService,
    notifications::{NotificationAdded, NotificationService},
    osd::OsdService,
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use std::collections::HashMap;
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
            Err(e) => Err(e.into()),
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
        eprintln!("Failed to initialize CEF: {e:?}");
        std::process::exit(1);
    }
    eprintln!("CEF initialized successfully!");
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
                if send_dbus_command("ToggleChat") {
                    std::process::exit(0);
                } else {
                    eprintln!("nwidgets is not running or D-Bus call failed");
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
            DbusService::init(cx);
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
                        origin: Point {
                            x: px(0.0),
                            y: px(0.0),
                        },
                        size: Size {
                            width: px(3440.0),
                            height: px(50.0),
                        },
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
            )
            .unwrap();

            // Chat window - NOT opened at startup
            let chat_pinned: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
            let chat_window: Arc<Mutex<Option<WindowHandle<ChatWidget>>>> =
                Arc::new(Mutex::new(None));
            let chat_window_clone = Arc::clone(&chat_window);
            let chat_window_pin = Arc::clone(&chat_window);
            let chat_pinned_toggle = Arc::clone(&chat_pinned);
            let chat_pinned_pin = Arc::clone(&chat_pinned);

            let open_chat_window =
                |cx: &mut App, pinned: bool| -> Option<WindowHandle<ChatWidget>> {
                    let (layer, exclusive_zone) = if pinned {
                        (Layer::Top, Some(px(600.0)))
                    } else {
                        (Layer::Overlay, None)
                    };
                    cx.open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(Bounds {
                                origin: Point {
                                    x: px(0.0),
                                    y: px(0.0),
                                },
                                size: Size {
                                    width: px(600.0),
                                    height: px(1370.0),
                                },
                            })),
                            titlebar: None,
                            window_background: WindowBackgroundAppearance::Transparent,
                            kind: WindowKind::LayerShell(LayerShellOptions {
                                namespace: "nwidgets-chat".to_string(),
                                layer,
                                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
                                exclusive_zone,
                                margin: Some((px(50.0), px(0.0), px(0.0), px(0.0))),
                                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                                ..Default::default()
                            }),
                            app_id: Some("nwidgets-chat".to_string()),
                            ..Default::default()
                        },
                        |_window, cx| cx.new(ChatWidget::new),
                    )
                    .ok()
                };

            // Subscribe to chat toggle events
            cx.subscribe(&chat_service, move |_service, _event: &ChatToggled, cx| {
                let mut window = chat_window_clone.lock();
                let pinned = *chat_pinned_toggle.lock();
                if let Some(handle) = window.take() {
                    // Save URL before closing
                    let _ = handle.update(cx, |chat, _window, cx| {
                        if let Some(url) = chat.current_url(cx) {
                            widgets::chat::save_url(&url);
                        }
                    });
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                } else {
                    *window = open_chat_window(cx, pinned);
                }
            })
            .detach();

            // Subscribe to chat pin events - only if window is open and focused
            cx.subscribe(
                &chat_service,
                move |_service, event: &ChatPinToggled, cx| {
                    let mut window = chat_window_pin.lock();
                    if let Some(handle) = window.as_ref() {
                        let is_focused = handle
                            .update(cx, |_, window, cx| window.focused(cx).is_some())
                            .unwrap_or(false);
                        if is_focused {
                            *chat_pinned_pin.lock() = event.pinned;
                            // Save current URL before closing
                            let _ = handle.update(cx, |chat, _window, cx| {
                                if let Some(url) = chat.current_url(cx) {
                                    widgets::chat::save_url(&url);
                                }
                            });
                            let handle = window.take().unwrap();
                            let _ = handle.update(cx, |_, window, _| window.remove_window());
                            *window = open_chat_window(cx, event.pinned);
                        }
                    }
                },
            )
            .detach();

            // Subscribe to chat navigate events
            let chat_window_nav = Arc::clone(&chat_window);
            cx.subscribe(
                &chat_service,
                move |_service, event: &ChatNavigate, cx| {
                    let window = chat_window_nav.lock();
                    if let Some(handle) = window.as_ref() {
                        let _ = handle.update(cx, |chat, _window, cx| {
                            chat.navigate(&event.url, cx);
                        });
                    }
                },
            )
            .detach();

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
                        )
                        .detach();
                    }
                },
            )
            .detach();

            cx.activate(true);
        });
}
