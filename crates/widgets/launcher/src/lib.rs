use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub const BAR_HEIGHT: f32 = 50.0;
pub const LAUNCHER_WIDTH: f32 = 700.0;
pub const LAUNCHER_HEIGHT: f32 = 482.0;

/// Ouvre la fenêtre layer shell pour le lanceur d'applications (masquée par défaut).
pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> anyhow::Result<WindowHandle<T>> {
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(BAR_HEIGHT) },
                size: Size { width: px(1.0), height: px(1.0) },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-launcher".to_string(),
                layer: Layer::Background,
                anchor: Anchor::TOP,
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            window.set_input_region(Some(&[]));
            build_view(window, cx)
        },
    )?;

    Ok(window)
}

/// Bascule la visibilité de la fenêtre du lanceur.
pub fn set_visible<T: 'static>(
    handle: &WindowHandle<T>,
    visible: bool,
    focus_handle: Option<&gpui::FocusHandle>,
    cx: &mut App,
) {
    let _ = handle.update(cx, |_, window, cx| {
        if visible {
            window.set_layer(Layer::Overlay);
            window.set_input_region(Some(&[Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(LAUNCHER_WIDTH), px(LAUNCHER_HEIGHT)),
            }]));
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            window.resize(size(px(LAUNCHER_WIDTH), px(LAUNCHER_HEIGHT)));
            if let Some(fh) = focus_handle {
                window.focus(fh, cx);
            }
            cx.activate(true);
        } else {
            window.set_layer(Layer::Background);
            window.set_input_region(Some(&[]));
            window.set_keyboard_interactivity(KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
    });
}
