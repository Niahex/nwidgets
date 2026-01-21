use gpui::*;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::services::{
    audio::AudioService, bluetooth::BluetoothService, cef::CefService,
    chat::{ChatNavigate, ChatService, ChatToggled}, clipboard::ClipboardMonitor,
    control_center::ControlCenterService, dbus::DbusService,
    hyprland::{FullscreenChanged, HyprlandService, WorkspaceChanged},
    launcher::{LauncherService, LauncherToggled}, mpris::MprisService,
    notifications::{NotificationAdded, NotificationService}, osd::OsdService,
    pomodoro::PomodoroService, system_monitor::SystemMonitorService, systray::SystrayService,
};

use crate::widgets::{
    chat::ChatWidget, launcher::LauncherWidget,
    notifications::{NotificationsStateChanged, NotificationsWindowManager}, panel::Panel,
};

pub fn run(cx: &mut App) {
    bind_keys(cx);
    let (clipboard_monitor, _osd_service) = initialize_services(cx);
    
    let (chat_service, launcher_service, notif_service) = get_services(cx);
    
    open_panel(cx);
    let chat_window = open_chat(cx);
    let launcher_window = open_launcher(cx, launcher_service.clone(), clipboard_monitor);
    
    setup_subscriptions(cx, chat_service, launcher_service, notif_service, chat_window, launcher_window);
    
    cx.activate(true);
}

fn bind_keys(cx: &mut App) {
    use crate::widgets::control_center::CloseControlCenter;
    use crate::widgets::launcher::{Backspace, Down, Launch, Quit, Up};

    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("up", Up, None),
        KeyBinding::new("down", Down, None),
        KeyBinding::new("enter", Launch, None),
        KeyBinding::new("escape", Quit, None),
        KeyBinding::new("escape", CloseControlCenter, None),
    ]);
}

fn initialize_services(cx: &mut App) -> (Entity<ClipboardMonitor>, Entity<OsdService>) {
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
    ChatService::init(cx);
    LauncherService::init(cx);
    let clipboard_monitor = ClipboardMonitor::init(cx);
    NotificationService::init(cx);
    let osd_service = OsdService::init(cx);
    ControlCenterService::init(cx);
    crate::services::cef::CefService::init(cx);
    
    (clipboard_monitor, osd_service)
}

fn get_services(cx: &mut App) -> (
    Entity<ChatService>,
    Entity<LauncherService>,
    Entity<NotificationService>,
) {
    (
        ChatService::global(cx),
        LauncherService::global(cx),
        NotificationService::global(cx),
    )
}

fn open_panel(cx: &mut App) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(0.0) },
                size: Size { width: px(3440.0), height: px(68.0) },
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
    .expect("Failed to create panel window");
}

fn open_chat(cx: &mut App) -> Arc<Mutex<WindowHandle<ChatWidget>>> {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let window = cx
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

    Arc::new(Mutex::new(window))
}

fn open_launcher(
    cx: &mut App,
    launcher_service: Entity<LauncherService>,
    clipboard_monitor: Entity<ClipboardMonitor>,
) -> Arc<Mutex<WindowHandle<LauncherWidget>>> {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point { x: px(0.0), y: px(0.0) },
                    size: Size { width: px(1.0), height: px(1.0) },
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
                    LauncherWidget::new(cx, launcher_service, clipboard_monitor)
                })
            },
        )
        .expect("Failed to create launcher window");

    Arc::new(Mutex::new(window))
}

fn setup_subscriptions(
    cx: &mut App,
    chat_service: Entity<ChatService>,
    launcher_service: Entity<LauncherService>,
    notif_service: Entity<NotificationService>,
    chat_window: Arc<Mutex<WindowHandle<ChatWidget>>>,
    launcher_window: Arc<Mutex<WindowHandle<LauncherWidget>>>,
) {
    setup_chat_subscriptions(cx, chat_service, chat_window);
    setup_launcher_subscriptions(cx, launcher_service, launcher_window);
    setup_notification_subscriptions(cx, notif_service);
}

fn setup_chat_subscriptions(
    cx: &mut App,
    chat_service: Entity<ChatService>,
    chat_window: Arc<Mutex<WindowHandle<ChatWidget>>>,
) {
    let chat_window_toggle = Arc::clone(&chat_window);
    let chat_window_nav = Arc::clone(&chat_window);
    let chat_service2 = chat_service.clone();
    let chat_service3 = chat_service.clone();

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
                window.set_exclusive_edge(gpui::layer_shell::Anchor::LEFT);
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
                    crate::widgets::chat::save_url(&url);
                }
                window.set_exclusive_zone(0);
                window.resize(size(px(1.0), px(1.0)));
                window.set_layer(gpui::layer_shell::Layer::Background);
            }
            cx.notify();
        });
    })
    .detach();

    cx.subscribe(
        &HyprlandService::global(cx),
        move |_hypr, event: &FullscreenChanged, cx| {
            if chat_service2.read(cx).visible && event.0 {
                chat_service2.update(cx, |cs, cx| cs.toggle(cx));
            }
        },
    )
    .detach();

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

    cx.subscribe(&chat_service, move |_service, event: &ChatNavigate, cx| {
        let window = chat_window_nav.lock();
        let _ = window.update(cx, |chat, _window, cx| {
            chat.navigate(&event.url, cx);
        });
    })
    .detach();
}

fn setup_launcher_subscriptions(
    cx: &mut App,
    launcher_service: Entity<LauncherService>,
    launcher_window: Arc<Mutex<WindowHandle<LauncherWidget>>>,
) {
    cx.subscribe(
        &launcher_service,
        move |service, _event: &LauncherToggled, cx| {
            let window = launcher_window.lock();
            let visible = service.read(cx).visible;
            let _ = window.update(cx, |launcher, window, cx| {
                if visible {
                    window.resize(size(px(700.0), px(500.0)));
                    window.set_keyboard_interactivity(
                        gpui::layer_shell::KeyboardInteractivity::Exclusive,
                    );
                    window.set_layer(gpui::layer_shell::Layer::Overlay);
                    launcher.reset();
                    window.focus(launcher.focus_handle(), cx);
                    cx.activate(true);
                } else {
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
}

fn setup_notification_subscriptions(
    cx: &mut App,
    notif_service: Entity<NotificationService>,
) {
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
}
