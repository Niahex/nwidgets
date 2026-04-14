use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::widgets::r#macro::{MacroService, MacroToggled, MacroWidget};

static MACRO_WINDOW: OnceLock<Arc<Mutex<WindowHandle<MacroWidget>>>> = OnceLock::new();

pub fn open(cx: &mut App) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let macro_service = MacroService::global(cx);

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
                    namespace: "nwidgets-macro".to_string(),
                    layer: Layer::Background,
                    anchor: Anchor::empty(),
                    exclusive_zone: None,
                    margin: None,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                app_id: Some("nwidgets-macro".to_string()),
                is_movable: false,
                ..Default::default()
            },
            move |_window, cx| cx.new(|cx| MacroWidget::new(cx, macro_service)),
        )
        .expect("Failed to create macro window");

    MACRO_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn on_toggle(service: Entity<MacroService>, _event: &MacroToggled, cx: &mut App) {
    let Some(window_arc) = MACRO_WINDOW.get() else {
        return;
    };
    let visible = service.read(cx).visible();
    let window = window_arc.lock();

    if let Err(err) = window.update(cx, |macro_widget, window, cx| {
        if visible {
            window.set_layer(gpui::layer_shell::Layer::Overlay);
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::OnDemand);
            window.resize(size(px(800.0), px(600.0)));
            window.focus(macro_widget.focus_handle(), cx);
            cx.activate(true);
        } else {
            window.set_layer(gpui::layer_shell::Layer::Background);
            window.set_input_region(None);
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
        cx.notify();
    }) {
        log::error!("Failed to update macro window: {err}");
    }
}
