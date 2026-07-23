use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

/// Rayon des coins arrondis sous la barre.
pub const CORNER_RADIUS: f32 = 12.0;
pub const BAR_HEIGHT: f32 = 50.0;

/// Ouvre la fenêtre layer shell pour le panneau/bar.
pub fn open<T: gpui::Render + 'static>(
    cx: &mut App,
    build_view: impl FnOnce(&mut Window, &mut App) -> Entity<T>,
) -> anyhow::Result<WindowHandle<T>> {
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(0.0) },
                size: Size {
                    width: px(3440.0),
                    height: px(350.0), // Surface height expanded for GPUI popovers & context menus
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(px(BAR_HEIGHT)),
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            // Restreindre la zone de clic à la barre uniquement (50px + 12px de coins = 62px)
            // afin que la zone transparente sous la barre ne bloque pas les clics sur les autres applications !
            window.set_input_region(Some(&[Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(3440.0), px(BAR_HEIGHT + CORNER_RADIUS)),
            }]));
            build_view(window, cx)
        },
    )?;

    Ok(window)
}
