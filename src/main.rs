use gpui::{
    App, Application, Bounds, WindowBounds, WindowOptions, WindowKind,
    WindowBackgroundAppearance, prelude::*, px, point, Size,
};

use gpui::layer_shell::{LayerShellOptions, Layer, Anchor, KeyboardInteractivity};

mod shell;
mod modules;

use shell::Shell;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Background layer (wallpaper) - ajusté pour éviter panel et drawers
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(60.), px(25.)),
                    size: Size::new(px(3355.), px(1390.)),
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

        // Top drawer
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(60.), px(0.)),
                    size: Size::new(px(3380.), px(25.)),
                })),
                app_id: Some("nwidgets-drawer-top".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-drawer-top".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(px(25.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_drawer_top),
        ).unwrap();

        // Bottom drawer
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(60.), px(1415.)),
                    size: Size::new(px(3380.), px(25.)),
                })),
                app_id: Some("nwidgets-drawer-bottom".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-drawer-bottom".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(px(25.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_drawer_bottom),
        ).unwrap();

        // Right drawer
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(3415.), px(25.)),
                    size: Size::new(px(25.), px(1390.)),
                })),
                app_id: Some("nwidgets-drawer-right".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-drawer-right".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::RIGHT,
                    exclusive_zone: Some(px(25.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Shell::new_drawer_right),
        ).unwrap();

        cx.activate(true);
    });
}
