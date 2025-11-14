use gpui::{
    App, Application, Bounds, WindowBounds, WindowOptions, WindowKind,
    WindowBackgroundAppearance, prelude::*, px, point, Size,
};

use gpui::layer_shell::{LayerShellOptions, Layer, Anchor, KeyboardInteractivity};

mod shell;
mod modules;
mod osd;

use shell::Shell;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Background layer (wallpaper) - plein écran sauf panel
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(60.), px(0.)),
                    size: Size::new(px(3380.), px(1440.)),
                })),
                app_id: Some("nwidgets-background".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-background".to_string(),
                    layer: Layer::Background,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_background),
        ).unwrap();

        // Left panel (sidebar)
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(60.), px(1440.)),
                })),
                app_id: Some("nwidgets-panel".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-panel".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM,
                    exclusive_zone: Some(px(60.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_panel),
        ).unwrap();

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
            |_, cx| cx.new(|_| osd::Osd::new(osd::OsdType::Volume(50))),
        ).unwrap();

        // Notifications
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(3040.), px(20.)),
                    size: Size::new(px(380.), px(1400.)),
                })),
                app_id: Some("nwidgets-notifications".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-notifications".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::RIGHT,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_notifications),
        ).unwrap();

        cx.activate(true);
    });
}
