use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub const BAR_HEIGHT: f32 = 50.0;

/// Ouvre la fenêtre layer shell pour le lanceur d'applications.
pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> anyhow::Result<WindowHandle<T>> {
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(BAR_HEIGHT) },
                size: Size { width: px(700.0), height: px(500.0) },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-launcher".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP,
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| build_view(window, cx),
    )?;

    Ok(window)
}

/// Bascule la visibilité de la fenêtre du lanceur.
pub fn on_toggle<T: 'static>(
    handle: &WindowHandle<T>,
    visible: bool,
    focus_handle: Option<&gpui::FocusHandle>,
    cx: &mut App,
) {
    let _ = handle.update(cx, |_, window, cx| {
        if visible {
            window.set_layer(gpui::layer_shell::Layer::Overlay);
            window.set_keyboard_interactivity(
                gpui::layer_shell::KeyboardInteractivity::Exclusive,
            );
            window.resize(gpui::size(px(700.0), px(500.0)));
            if let Some(fh) = focus_handle {
                window.focus(fh, cx);
            }
            cx.activate(true);
        } else {
            window.set_layer(gpui::layer_shell::Layer::Background);
            window.set_input_region(None);
            window.set_keyboard_interactivity(
                gpui::layer_shell::KeyboardInteractivity::None,
            );
            window.resize(gpui::size(px(1.0), px(1.0)));
        }
    });
}
