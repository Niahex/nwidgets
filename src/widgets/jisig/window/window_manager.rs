use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::widgets::jisig::JisigWidget;

static JISIG_WINDOW: OnceLock<Arc<Mutex<WindowHandle<JisigWidget>>>> =
    OnceLock::new();

pub fn open(cx: &mut App) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let window = cx
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
                    namespace: "nwidgets-jisig".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::RIGHT,
                    exclusive_zone: None,
                    margin: Some((px(40.0), px(10.0), px(20.0), px(0.0))),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                app_id: Some("nwidgets-jisig".to_string()),
                is_movable: false,
                ..Default::default()
            },
            |_window, cx| cx.new(JisigWidget::new),
        )
        .expect("Failed to create jisig window");

    JISIG_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn get_window() -> Option<Arc<Mutex<WindowHandle<JisigWidget>>>> {
    JISIG_WINDOW.get().cloned()
}
