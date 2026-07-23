use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> Option<WindowHandle<T>> {
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(0.0) },
                size: Size { width: px(1.0), height: px(1.0) },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-notifications".to_string(),
                layer: Layer::Background,
                anchor: Anchor::TOP | Anchor::RIGHT,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            let entity = build_view(window, cx);
            window.set_input_region(Some(&[]));
            window.resize(size(px(1.0), px(1.0)));
            entity
        },
    )
    .ok()
}

pub fn set_visible<T: 'static>(handle: &WindowHandle<T>, visible: bool, height_px: f32, cx: &mut App) {
    let _ = handle.update(cx, |_, window, _| {
        if visible {
            window.set_layer(Layer::Overlay);
            window.set_input_region(Some(&[Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(380.0), px(height_px)),
            }]));
            window.resize(size(px(380.0), px(height_px)));
        } else {
            window.set_layer(Layer::Background);
            window.set_input_region(Some(&[]));
            window.set_keyboard_interactivity(KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
    });
}
