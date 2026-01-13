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
    control_center::{ControlCenterService, ControlCenterStateChanged},
    dbus::DbusService,
    discord::{DiscordService, DiscordToggled},
    hyprland::{FullscreenChanged, HyprlandService, WorkspaceChanged},
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
    discord::DiscordWidget,
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
            let discord_service = DiscordService::init(cx);
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
            .unwrap();

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
                .unwrap();

            let chat_window_arc: Arc<Mutex<WindowHandle<ChatWidget>>> =
                Arc::new(Mutex::new(chat_window));
            let chat_window_toggle = Arc::clone(&chat_window_arc);
            let chat_window_fs = Arc::clone(&chat_window_arc);
            let chat_window_ws = Arc::clone(&chat_window_arc);
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
                        window.set_margin(if fullscreen { 0 } else { 40 }, 0, if fullscreen { 0 } else { 20 }, if fullscreen { 0 } else { 10 });
                        window.set_exclusive_edge(Anchor::LEFT);
                        window.set_exclusive_zone(if fullscreen { 0 } else { 600 });
                    } else {
                        if let Some(url) = chat.current_url(cx) {
                            widgets::chat::save_url(&url);
                        }
                        window.set_exclusive_zone(0);
                        window.resize(size(px(1.0), px(1.0)));
                    }
                    cx.notify();
                });
            })
            .detach();

            // Close chat when entering fullscreen
            cx.subscribe(&HyprlandService::global(cx), move |_hypr, event: &FullscreenChanged, cx| {
                if chat_service2.read(cx).visible && event.0 {
                    chat_service2.update(cx, |cs, cx| cs.toggle(cx));
                }
            })
            .detach();

            // Close chat when switching to fullscreen workspace
            cx.subscribe(&HyprlandService::global(cx), move |_hypr, _event: &WorkspaceChanged, cx| {
                let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
                if chat_service3.read(cx).visible && fullscreen {
                    chat_service3.update(cx, |cs, cx| cs.toggle(cx));
                }
            })
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

            // Discord window - created at startup, starts hidden (1x1), CEF lazy-loaded on first toggle
            let discord_window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Point { x: px(0.0), y: px(0.0) },
                            size: Size { width: px(1.0), height: px(1.0) },
                        })),
                        titlebar: None,
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(WindowDecorations::Client),
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nwidgets-discord".to_string(),
                            layer: Layer::Overlay,
                            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::RIGHT,
                            exclusive_zone: None,
                            margin: Some((px(40.0), px(10.0), px(20.0), px(0.0))),
                            keyboard_interactivity: KeyboardInteractivity::OnDemand,
                            ..Default::default()
                        }),
                        app_id: Some("nwidgets-discord".to_string()),
                        is_movable: false,
                        ..Default::default()
                    },
                    |_window, cx| cx.new(DiscordWidget::new),
                )
                .unwrap();

            let discord_window_arc: Arc<Mutex<WindowHandle<DiscordWidget>>> =
                Arc::new(Mutex::new(discord_window));

            let discord_window_arc2 = Arc::clone(&discord_window_arc);
            let discord_window_arc3 = Arc::clone(&discord_window_arc);
            let discord_window_arc4 = Arc::clone(&discord_window_arc);
            let discord_service2 = discord_service.clone();
            let discord_service3 = discord_service.clone();
            let discord_service4 = discord_service.clone();
            let hyprland_service = HyprlandService::global(cx);
            let cc_service = ControlCenterService::global(cx);

            // Discord toggle handler - also close control center if open
            let cc_service2 = cc_service.clone();
            cx.subscribe(&discord_service, move |service, _event: &DiscordToggled, cx| {
                let window = discord_window_arc.lock();
                let visible = service.read(cx).visible;
                let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
                
                // Close control center when opening Discord
                if visible && cc_service2.read(cx).is_visible() {
                    cc_service2.update(cx, |cc, cx| cc.toggle(cx));
                }
                
                let _ = window.update(cx, |discord, window, cx| {
                    if visible {
                        let height = if fullscreen { 1440 } else { 1370 };
                        window.resize(size(px(1500.0), px(height as f32)));
                        discord.resize_browser(1500, height, cx);
                        window.set_margin(if fullscreen { 0 } else { 40 }, if fullscreen { 0 } else { 10 }, if fullscreen { 0 } else { 20 }, 0);
                        window.set_exclusive_edge(Anchor::RIGHT);
                        window.set_exclusive_zone(if fullscreen { 0 } else { 1500 });
                    } else {
                        window.set_exclusive_zone(0);
                        window.resize(size(px(1.0), px(1.0)));
                    }
                    cx.notify();
                });
            })
            .detach();

            // Hide Discord when control center opens
            cx.subscribe(&cc_service, move |_cc, _event: &ControlCenterStateChanged, cx| {
                let discord_visible = discord_service3.read(cx).visible;
                let cc_visible = ControlCenterService::global(cx).read(cx).is_visible();
                let window = discord_window_arc3.lock();
                
                if discord_visible && cc_visible {
                    let _ = window.update(cx, |_discord, window, cx| {
                        window.set_exclusive_zone(0);
                        window.resize(size(px(1.0), px(1.0)));
                        cx.notify();
                    });
                }
            })
            .detach();

            // Close Discord when entering fullscreen workspace (if already open)
            cx.subscribe(&hyprland_service, move |_hypr, event: &FullscreenChanged, cx| {
                let discord_visible = discord_service2.read(cx).visible;
                
                if discord_visible && event.0 {
                    // Fullscreen detected while Discord open → close Discord
                    discord_service2.update(cx, |ds, cx| ds.toggle(cx));
                }
            })
            .detach();

            // Close Discord when switching to workspace with fullscreen
            cx.subscribe(&HyprlandService::global(cx), move |_hypr, _event: &WorkspaceChanged, cx| {
                let discord_visible = discord_service4.read(cx).visible;
                let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
                
                if discord_visible && fullscreen {
                    // Switched to fullscreen workspace → close Discord
                    discord_service4.update(cx, |ds, cx| ds.toggle(cx));
                }
            })
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
