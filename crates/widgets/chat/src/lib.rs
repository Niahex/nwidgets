use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;

pub const BAR_HEIGHT: f32 = 50.0;
pub const CHAT_WIDTH: f32 = 600.0;

/// Ouvre la fenêtre layer shell pour le chat (masquée par défaut).
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
                namespace: "nwidgets-chat".to_string(),
                layer: Layer::Background,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            app_id: Some("nwidgets-chat".to_string()),
            is_movable: false,
            ..Default::default()
        },
        |window, cx| {
            window.set_input_region(Some(&[]));
            build_view(window, cx)
        },
    )?;

    Ok(window)
}

/// Bascule l'affichage du chat.
pub fn set_visible<T: 'static>(handle: &WindowHandle<T>, visible: bool, cx: &mut App) {
    let _ = handle.update(cx, |_, window, cx| {
        if visible {
            window.set_layer(Layer::Overlay);
            window.set_input_region(Some(&[Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(CHAT_WIDTH), px(2000.0)),
            }]));
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
            window.resize(size(px(CHAT_WIDTH), px(2000.0)));
            cx.activate(true);
        } else {
            window.set_layer(Layer::Background);
            window.set_input_region(Some(&[]));
            window.set_keyboard_interactivity(KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
    });
}
