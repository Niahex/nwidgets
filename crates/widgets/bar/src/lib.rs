use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

/// Rayon des coins arrondis sous la barre.
pub const CORNER_RADIUS: f32 = 12.0;

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
                    height: px(50.0 + CORNER_RADIUS),
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(px(50.0)),
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| build_view(window, cx),
    )?;

    Ok(window)
}
