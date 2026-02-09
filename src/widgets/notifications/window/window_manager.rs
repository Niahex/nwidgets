use crate::widgets::notifications::widget::NotificationsWidget;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub struct NotificationsWindowManager {
    window: Option<WindowHandle<NotificationsWidget>>,
}

impl NotificationsWindowManager {
    pub fn new() -> Self {
        Self { window: None }
    }

    pub fn open_window(&mut self, cx: &mut App) -> Option<Entity<NotificationsWidget>> {
        if let Some(window) = &self.window {
            let _ = window.update(cx, |_, window, _| {
                window.set_layer(Layer::Overlay);
                window.resize(size(px(400.0), px(600.0)));
            });
            return cx.read_window(window, |entity, _| entity.clone()).ok();
        }

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px(3040.0),
                            y: px(60.0),
                        },
                        size: Size {
                            width: px(400.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-notifications".to_string(),
                        layer: Layer::Overlay,
                        anchor: Anchor::TOP | Anchor::RIGHT,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(NotificationsWidget::new)
                },
            )
            .ok()?;

        let entity = cx.read_window(&window, |entity, _| entity.clone()).ok();
        self.window = Some(window);
        entity
    }

    pub fn close_window(&mut self, cx: &mut App) {
        if let Some(window) = &self.window {
            let _ = window.update(cx, |_, window, _| {
                window.set_layer(Layer::Background);
                window.set_input_region(None);
                window.set_keyboard_interactivity(KeyboardInteractivity::None);
                window.resize(size(px(1.0), px(1.0)));
            });
        }
    }
}
