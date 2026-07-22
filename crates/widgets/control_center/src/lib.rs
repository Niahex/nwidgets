use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

/// Hauteur de la barre en haut de l'écran (réservée par le panneau).
pub const BAR_HEIGHT: f32 = 50.0;

/// Ouvre la fenêtre layer shell pour le panneau de contrôle.
/// Prend tout l'écran sauf les 50px réservés à la barre en haut.
pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> anyhow::Result<WindowHandle<T>> {
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(BAR_HEIGHT) },
                size: Size {
                    width: px(600.0),
                    height: px(2000.0), // étiré en hauteur par l'ancrage BOTTOM
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-control-center".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::RIGHT | Anchor::BOTTOM,
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
