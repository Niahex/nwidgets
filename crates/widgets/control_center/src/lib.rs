use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

/// Hauteur de la barre en haut de l'écran (réservée par le panneau).
pub const BAR_HEIGHT: f32 = 50.0;
pub const CONTROL_CENTER_WIDTH: f32 = 600.0;

/// Ouvre la fenêtre layer shell pour le panneau de contrôle (masquée par défaut).
pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> anyhow::Result<WindowHandle<T>> {
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(BAR_HEIGHT) },
                size: Size {
                    width: px(1.0),
                    height: px(1.0),
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-control-center".to_string(),
                layer: Layer::Background,
                anchor: Anchor::TOP | Anchor::RIGHT | Anchor::BOTTOM,
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            window.set_input_region(None);
            build_view(window, cx)
        },
    )?;

    Ok(window)
}

/// Bascule l'affichage du control center.
pub fn set_visible<T: 'static>(handle: &WindowHandle<T>, visible: bool, cx: &mut App) {
    let _ = handle.update(cx, |_, window, cx| {
        if visible {
            window.set_layer(Layer::Overlay);
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            window.resize(size(px(CONTROL_CENTER_WIDTH), px(2000.0)));
            cx.activate(true);
        } else {
            window.set_layer(Layer::Background);
            window.set_input_region(None);
            window.set_keyboard_interactivity(KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
    });
}

/// Bascule la visibilité du control center depuis un AnyWindowHandle.
pub fn toggle(handle: &AnyWindowHandle, visible: bool, cx: &mut App) {
    let _ = handle.update(cx, |_, window, cx| {
        if visible {
            window.set_layer(Layer::Overlay);
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            window.resize(size(px(CONTROL_CENTER_WIDTH), px(2000.0)));
            cx.activate(true);
        } else {
            window.set_layer(Layer::Background);
            window.set_input_region(None);
            window.set_keyboard_interactivity(KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
    });
}
