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
        // Background layer (wallpaper)
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(3440.), px(1440.)),
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

        cx.activate(true);
    });
}
