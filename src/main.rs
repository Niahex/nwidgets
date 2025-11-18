use gpui::{
    actions, point, prelude::*, px, App, Application, Bounds, KeyBinding, Size,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};

use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

mod modules;
mod services;
mod theme;
mod widgets;

use widgets::launcher;
use widgets::osd;
use widgets::{Launcher, Panel};

actions!(nwidgets, [OpenLauncher]);

fn main() {
    Application::new().run(|cx: &mut App| {
        // Keybindings globaux
        cx.bind_keys([KeyBinding::new("super-space", OpenLauncher, None)]);

        // Action pour ouvrir le launcher
        cx.on_action(|_: &OpenLauncher, cx| {
            let window = cx
                .open_window(
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
                )
                .unwrap();

            // Configurer les keybindings pour le launcher
            window
                .update(cx, |view, window, cx| {
                    cx.bind_keys([
                        KeyBinding::new("backspace", launcher::Backspace, None),
                        KeyBinding::new("up", launcher::Up, None),
                        KeyBinding::new("down", launcher::Down, None),
                        KeyBinding::new("enter", launcher::Launch, None),
                        KeyBinding::new("escape", launcher::Quit, None),
                    ]);

                    window.focus(&view.focus_handle);
                    cx.activate(true);
                })
                .unwrap();
        });

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
            |_, cx| cx.new(Panel::new),
        )
        .unwrap();

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
        )
        .unwrap();

        // Démarrer le service de notifications avec son manager
        use crate::services::notifications::NotificationManager;
        let _notification_manager = NotificationManager::new(cx);
        println!("[MAIN] 📢 Notification manager started");

        cx.activate(true);
    });
}
