use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

use crate::widgets::panel::Panel;

pub fn open(cx: &mut App) {
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
    ).expect("Failed to create panel window");
}
