use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::widgets::chat::ChatWidget;

static CHAT_WINDOW: OnceLock<Arc<Mutex<WindowHandle<ChatWidget>>>> =
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

    CHAT_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn get_window() -> Option<Arc<Mutex<WindowHandle<ChatWidget>>>> {
    CHAT_WINDOW.get().cloned()
}
