use gpui::{
    actions, App, Application, Bounds, WindowBounds, WindowOptions, WindowKind,
    WindowBackgroundAppearance, prelude::*, px, point, Size, KeyBinding,
};

use gpui::layer_shell::{LayerShellOptions, Layer, Anchor, KeyboardInteractivity};

mod shell;
mod modules;
mod services;
mod wayland_ext;

use shell::Shell;
use modules::launcher::Launcher;
use modules::launcher;
use modules::osd;
use services::NotificationManager;

actions!(nwidgets, [OpenLauncher]);

fn main() {
    Application::new().run(|cx: &mut App| {
        // Keybindings globaux
        cx.bind_keys([
            KeyBinding::new("super-space", OpenLauncher, None),
        ]);

        // Action pour ouvrir le launcher
        cx.on_action(|_: &OpenLauncher, cx| {
            let window = cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(760.), px(520.)),
                        size: Size::new(px(800.), px(400.)),
                    })),
                    app_id: Some("nwidgets-launcher".to_string()),
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-launcher".to_string(),
                        layer: Layer::Overlay,
                        anchor: Anchor::empty(),
                        keyboard_interactivity: KeyboardInteractivity::Exclusive,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(Launcher::new),
            ).unwrap();

            // Configurer les keybindings pour le launcher
            window.update(cx, |view, window, cx| {
                cx.bind_keys([
                    KeyBinding::new("backspace", launcher::Backspace, None),
                    KeyBinding::new("up", launcher::Up, None),
                    KeyBinding::new("down", launcher::Down, None),
                    KeyBinding::new("enter", launcher::Launch, None),
                    KeyBinding::new("escape", launcher::Quit, None),
                ]);
                
                window.focus(&view.focus_handle);
                cx.activate(true);
            }).unwrap();
        });

        // Background layer (wallpaper) - DÉSACTIVÉ
        /*
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(48.)),
                    size: Size::new(px(3440.), px(1392.)),
                })),
                app_id: Some("nwidgets-background".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-background".to_string(),
                    layer: Layer::Background,
                    anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_background),
        ).unwrap();
        */

        // Top panel (horizontal bar)
        cx.open_window(
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
            |_, cx| cx.new(Shell::new_panel),
        ).unwrap();

        // Left corner decorator (rounded corner below panel)
        // DÉSACTIVÉ TEMPORAIREMENT
        /*
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(48.)),
                    size: Size::new(px(48.), px(48.)),
                })),
                app_id: Some("nwidgets-corner-left".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-corner-left".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::LEFT,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Shell::new_corner(cx, CornerPosition::BottomLeft)),
        ).unwrap();

        // Right corner decorator (rounded corner below panel)
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(3392.), px(48.)),
                    size: Size::new(px(48.), px(48.)),
                })),
                app_id: Some("nwidgets-corner-right".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-corner-right".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::RIGHT,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Shell::new_corner(cx, CornerPosition::BottomRight)),
        ).unwrap();
        */

        // OSD
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(1690.), px(100.)),
                    size: Size::new(px(256.), px(64.)),
                })),
                app_id: Some("nwidgets-osd".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-osd".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| osd::Osd::new(osd::OsdType::CapsLock(false), cx)),
        ).unwrap();

        // Notifications - Géré par NotificationManager avec des fenêtres dynamiques
        NotificationManager::new(cx);

        cx.activate(true);
    });
}
