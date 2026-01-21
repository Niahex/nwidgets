mod components;
mod services;
mod theme;
mod utils;
mod widgets;

use anyhow::Result;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::prelude::*;
use gpui::*;
use gpui::{Bounds, Point, Size, WindowBounds, WindowDecorations};
use parking_lot::Mutex;
use services::{
    audio::AudioService,
    bluetooth::BluetoothService,
    cef::CefService,
    chat::{ChatNavigate, ChatService, ChatToggled},
    clipboard::ClipboardMonitor,
    control_center::ControlCenterService,
    dbus::DbusService,
    hyprland::{FullscreenChanged, HyprlandService, WorkspaceChanged},
    launcher::{LauncherService, LauncherToggled},
    mpris::MprisService,
    notifications::{NotificationAdded, NotificationService},
    osd::OsdService,
    pomodoro::PomodoroService,
    system_monitor::SystemMonitorService,
    systray::SystrayService,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use widgets::{
    chat::ChatWidget,
    launcher::LauncherWidget,
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

fn main() {
    // Initialize logger with custom format, colors, and filters
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("blade_graphics", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .filter_module("zbus", log::LevelFilter::Warn)
        .filter_module("gpui::platform", log::LevelFilter::Warn)
        .filter_module("gpui::window", log::LevelFilter::Off)
        .format(|buf, record| {
            use std::io::Write;
            
            // ANSI color codes
            let level_str = match record.level() {
                log::Level::Error => "\x1b[1;31mERROR\x1b[0m", // Red bold
                log::Level::Warn => "\x1b[1;33mWARN \x1b[0m",  // Yellow bold
                log::Level::Info => "\x1b[32mINFO \x1b[0m",    // Green
                log::Level::Debug => "\x1b[36mDEBUG\x1b[0m",   // Cyan
                log::Level::Trace => "\x1b[37mTRACE\x1b[0m",   // White
            };
            
            // Extract module path and format it
            let module = record.module_path().unwrap_or("unknown");
            let category = if module.starts_with("nwidgets::services") {
                format!("service::{}", module.strip_prefix("nwidgets::services::").unwrap_or(module))
            } else if module.starts_with("nwidgets::widgets") {
                format!("widget::{}", module.strip_prefix("nwidgets::widgets::").unwrap_or(module))
            } else if module.starts_with("nwidgets::components") {
                format!("component::{}", module.strip_prefix("nwidgets::components::").unwrap_or(module))
            } else if module.starts_with("nwidgets") {
                module.strip_prefix("nwidgets::").unwrap_or(module).to_string()
            } else {
                module.to_string()
            };
            
            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_str,
                category,
                record.args()
            )
        })
        .init();

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
                    log::error!("nwidgets is not running or D-Bus call failed");
                    std::process::exit(1);
                }
            }
            _ => {
                log::error!("Unknown command: {}", args[1]);
                log::info!("Usage: nwidgets [chat]");
                std::process::exit(1);
            }
        }
    }

    // Determine assets path
    let assets_path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // Initialize GPUI FIRST, before CEF loads its EGL libs
    Application::new()
        .with_assets(Assets {
            base: assets_path,
            cache: parking_lot::RwLock::new(HashMap::new()),
        })
        .run(|cx: &mut App| {
            // Initialize CEF after GPUI has initialized its GPU context
            init_cef();

            gpui_tokio::init(cx);
            cx.set_global(theme::Theme::nord_dark());

            // Bind global keys for launcher
            use crate::widgets::launcher::{Backspace, Down, Launch, Quit, Up};
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, None),
                KeyBinding::new("up", Up, None),
                KeyBinding::new("down", Down, None),
                KeyBinding::new("enter", Launch, None),
                KeyBinding::new("escape", Quit, None),
            ]);

            // Bind global keys for control center
            use crate::widgets::control_center::CloseControlCenter;
            cx.bind_keys([KeyBinding::new("escape", CloseControlCenter, None)]);

            // Initialize global services
            HyprlandService::init(cx);
            AudioService::init(cx);
            BluetoothService::init(cx);
            crate::services::network::init_network_services(cx);
            SystemMonitorService::init(cx);
            MprisService::init(cx);
            PomodoroService::init(cx);
            SystrayService::init(cx);
            CefService::init(cx);
            DbusService::init(cx);
            let chat_service = ChatService::init(cx);
            let launcher_service = LauncherService::init(cx);
            let clipboard_monitor = ClipboardMonitor::init(cx);
            let notif_service = NotificationService::init(cx);
            let osd_service = OsdService::init(cx);
            ControlCenterService::init(cx);

            // Initialize CEF Service
            services::cef::CefService::init(cx);

            // Panel window
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px(0.0),
                            y: px(0.0),
                        },
                        size: Size {
                            width: px(3440.0),
                            height: px(68.0), // 50 + 18 for corners
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-panel".to_string(),
                        layer: Layer::Top,
                        anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                        exclusive_zone: Some(px(50.)), // Only reserve 50px
                        margin: None,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(Panel::new),
            )
            .expect("Failed to create panel window");

            // Chat window - created at startup, starts hidden (1x1)
            let chat_window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Point {
                                x: px(0.0),
                                y: px(0.0),
                            },
                            size: Size {
                                width: px(1.0),
                                height: px(1.0),
                            },
                        })),
                        titlebar: None,
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(WindowDecorations::Client),
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nwidgets-chat".to_string(),
                            layer: Layer::Overlay,
                            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
                            exclusive_zone: None,
                            margin: Some((px(40.0), px(0.0), px(20.0), px(10.0))),
                            keyboard_interactivity: KeyboardInteractivity::OnDemand,
                            ..Default::default()
                        }),
                        app_id: Some("nwidgets-chat".to_string()),
                        is_movable: false,
                        ..Default::default()
                    },
                    |_window, cx| cx.new(ChatWidget::new),
                )
                .expect("Failed to create chat window");

            // Launcher window - created at startup, starts hidden (1x1)
            let launcher_service_clone = launcher_service.clone();
            let clipboard_monitor_clone = clipboard_monitor.clone();
            let launcher_window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Point {
                                x: px(0.0),
                                y: px(0.0),
                            },
                            size: Size {
                                width: px(1.0),
                                height: px(1.0),
                            },
                        })),
                        titlebar: None,
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nwidgets-launcher".to_string(),
                            layer: Layer::Overlay,
                            anchor: Anchor::empty(),
                            exclusive_zone: None,
                            margin: None,
                            keyboard_interactivity: KeyboardInteractivity::None,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    move |_window, cx| {
                        cx.new(|cx| {
                            LauncherWidget::new(
                                cx,
                                launcher_service_clone.clone(),
                                clipboard_monitor_clone.clone(),
                            )
                        })
                    },
                )
                .expect("Failed to create launcher window");

            let chat_window_arc: Arc<Mutex<WindowHandle<ChatWidget>>> =
                Arc::new(Mutex::new(chat_window));
            let launcher_window_arc: Arc<Mutex<WindowHandle<LauncherWidget>>> =
                Arc::new(Mutex::new(launcher_window));
            let chat_window_toggle = Arc::clone(&chat_window_arc);
            let launcher_window_toggle = Arc::clone(&launcher_window_arc);
            let _chat_window_fs = Arc::clone(&chat_window_arc);
            let _chat_window_ws = Arc::clone(&chat_window_arc);
            let chat_service2 = chat_service.clone();
            let chat_service3 = chat_service.clone();

            // Subscribe to chat toggle events
            cx.subscribe(&chat_service, move |service, _event: &ChatToggled, cx| {
                let window = chat_window_toggle.lock();
                let visible = service.read(cx).visible;
                let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
                let _ = window.update(cx, |chat, window, cx| {
                    if visible {
                        let height = if fullscreen { 1440 } else { 1370 };
                        window.resize(size(px(600.0), px(height as f32)));
                        chat.resize_browser(600, height, cx);
                        chat.focus_input(cx);
                        window.set_margin(
                            if fullscreen { 0 } else { 40 },
                            0,
                            if fullscreen { 0 } else { 20 },
                            if fullscreen { 0 } else { 10 },
                        );
                        window.set_exclusive_edge(Anchor::LEFT);
                        window.set_exclusive_zone(if fullscreen { 0 } else { 600 });
                        window.set_layer(if fullscreen {
                            gpui::layer_shell::Layer::Overlay
                        } else {
                            gpui::layer_shell::Layer::Top
                        });
                        window.set_keyboard_interactivity(
                            gpui::layer_shell::KeyboardInteractivity::OnDemand,
                        );
                        cx.activate(true);
                    } else {
                        if let Some(url) = chat.current_url(cx) {
                            widgets::chat::save_url(&url);
                        }
                        window.set_exclusive_zone(0);
                        window.resize(size(px(1.0), px(1.0)));
                        window.set_layer(gpui::layer_shell::Layer::Background);
                    }
                    cx.notify();
                });
            })
            .detach();

            // Close chat when entering fullscreen
            cx.subscribe(
                &HyprlandService::global(cx),
                move |_hypr, event: &FullscreenChanged, cx| {
                    if chat_service2.read(cx).visible && event.0 {
                        chat_service2.update(cx, |cs, cx| cs.toggle(cx));
                    }
                },
            )
            .detach();

            // Close chat when switching to fullscreen workspace
            cx.subscribe(
                &HyprlandService::global(cx),
                move |_hypr, _event: &WorkspaceChanged, cx| {
                    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
                    if chat_service3.read(cx).visible && fullscreen {
                        chat_service3.update(cx, |cs, cx| cs.toggle(cx));
                    }
                },
            )
            .detach();

            // Subscribe to chat navigate events
            let chat_window_nav = Arc::clone(&chat_window_arc);
            cx.subscribe(&chat_service, move |_service, event: &ChatNavigate, cx| {
                let window = chat_window_nav.lock();
                let _ = window.update(cx, |chat, _window, cx| {
                    chat.navigate(&event.url, cx);
                });
            })
            .detach();

            // Subscribe to launcher toggle events
            cx.subscribe(
                &launcher_service,
                move |service, _event: &LauncherToggled, cx| {
                    let window = launcher_window_toggle.lock();
                    let visible = service.read(cx).visible;
                    log::debug!("Launcher toggle event received, visible: {visible}");
                    let _ = window.update(cx, |launcher, window, cx| {
                        if visible {
                            log::debug!("Showing launcher window");
                            window.resize(size(px(700.0), px(500.0)));
                            window.set_keyboard_interactivity(
                                gpui::layer_shell::KeyboardInteractivity::Exclusive,
                            );
                            window.set_layer(gpui::layer_shell::Layer::Overlay);
                            // Reset and focus the launcher when it becomes visible
                            launcher.reset();
                            window.focus(launcher.focus_handle(), cx);
                            cx.activate(true);
                        } else {
                            log::debug!("Hiding launcher window");
                            window.resize(size(px(1.0), px(1.0)));
                            window.set_keyboard_interactivity(
                                gpui::layer_shell::KeyboardInteractivity::None,
                            );
                            window.set_input_region(None);
                            window.set_layer(gpui::layer_shell::Layer::Background);
                        }
                        cx.notify();
                    });
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
