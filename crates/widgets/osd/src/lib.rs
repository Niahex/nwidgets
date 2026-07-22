use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

/// Gestionnaire de fenêtre OSD (On-Screen Display).
pub struct OsdWindow {
    handle: Option<AnyWindowHandle>,
}

impl OsdWindow {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    pub fn open<T: gpui::Render + 'static>(
        &mut self,
        cx: &mut App,
        build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
    ) -> Option<WindowHandle<T>> {
        if self.handle.is_some() {
            return None;
        }

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(0.0), y: px(0.0) },
                        size: Size { width: px(1.0), height: px(1.0) },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-osd".to_string(),
                        layer: Layer::Background,
                        anchor: Anchor::BOTTOM,
                        margin: None,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let entity = build_view(window, cx);
                    window.set_input_region(None);
                    window.resize(size(px(1.0), px(1.0)));
                    entity
                },
            )
            .ok()?;

        let handle = window.into();
        self.handle = Some(handle);
        Some(window)
    }

    pub fn show(&self, cx: &mut App) {
        if let Some(handle) = &self.handle {
            let _ = handle.update(cx, |_, window, _| {
                window.set_layer(Layer::Overlay);
                window.set_input_region(None);
                window.resize(size(px(400.0), px(64.0)));
            });
        }
    }

    pub fn hide(&self, cx: &mut App) {
        if let Some(handle) = &self.handle {
            let _ = handle.update(cx, |_, window, _| {
                window.set_layer(Layer::Background);
                window.set_input_region(None);
                window.set_keyboard_interactivity(KeyboardInteractivity::None);
                window.resize(size(px(1.0), px(1.0)));
            });
        }
    }
}

impl Default for OsdWindow {
    fn default() -> Self {
        Self::new()
    }
}
