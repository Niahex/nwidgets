use crate::widgets::osd::widget::OsdWidget;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub struct OsdWindowManager {
    window_handle: Option<AnyWindowHandle>,
}

impl OsdWindowManager {
    pub fn new() -> Self {
        Self {
            window_handle: None,
        }
    }

    pub fn has_window(&self) -> bool {
        self.window_handle.is_some()
    }

    pub fn open_window(&mut self, cx: &mut App) {
        if self.window_handle.is_some() {
            return;
        }

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px(760.0),
                            y: px(1020.0),
                        },
                        size: Size {
                            width: px(400.0),
                            height: px(64.0),
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-osd".to_string(),
                        layer: Layer::Background,
                        anchor: Anchor::BOTTOM,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_input_region(None);
                    window.resize(size(px(1.0), px(1.0)));
                    cx.new(|cx| OsdWidget::new(cx, None, false))
                },
            )
            .ok();

        self.window_handle = window.map(|w| w.into());
    }

    pub fn show_window(&mut self, cx: &mut App) {
        if let Some(window_handle) = &self.window_handle {
            let _ = window_handle.update(cx, |_, window, _| {
                window.set_layer(Layer::Overlay);
                window.set_input_region(None);
                window.resize(size(px(400.0), px(64.0)));
            });
        }
    }

    pub fn hide_window(&mut self, cx: &mut App) {
        if let Some(window_handle) = &self.window_handle {
            let _ = window_handle.update(cx, |_, window, _| {
                window.set_layer(Layer::Background);
                window.set_input_region(None);
                window.set_keyboard_interactivity(KeyboardInteractivity::None);
                window.resize(size(px(1.0), px(1.0)));
            });
        }
    }
}
