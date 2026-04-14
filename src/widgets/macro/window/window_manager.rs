use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::widgets::r#macro::{MacroService, MacroToggled, MacroWidget};

static MACRO_WINDOW: OnceLock<Arc<Mutex<Option<WindowHandle<MacroWidget>>>>> = OnceLock::new();

pub fn open(cx: &mut App) {
    MACRO_WINDOW.get_or_init(|| Arc::new(Mutex::new(None)));
}

pub fn on_toggle(service: Entity<MacroService>, _event: &MacroToggled, cx: &mut App) {
    let Some(window_arc) = MACRO_WINDOW.get() else {
        return;
    };

    let visible = service.read(cx).visible();
    let mut window_lock = window_arc.lock();

    if visible {
        // Ouvrir la fenêtre si elle n'existe pas ou est fermée
        if window_lock.is_none() {
            let macro_service = MacroService::global(cx);

            let window = match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px(100.0),
                            y: px(100.0),
                        },
                        size: Size {
                            width: px(800.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Macro Manager".into()),
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    window_background: WindowBackgroundAppearance::Opaque,
                    window_decorations: Some(WindowDecorations::Server),
                    kind: WindowKind::Normal,
                    app_id: Some("nwidgets-macro".to_string()),
                    is_movable: true,
                    ..Default::default()
                },
                move |_window, cx| cx.new(|cx| MacroWidget::new(cx, macro_service)),
            ) {
                Ok(window) => window,
                Err(e) => {
                    log::error!("Failed to create macro window: {}", e);
                    return;
                }
            };

            *window_lock = Some(window);
        } else if let Some(window) = window_lock.as_ref() {
            let _ = window.update(cx, |macro_widget, window, cx| {
                window.focus(macro_widget.focus_handle(), cx);
                cx.activate(true);
            });
        }
    } else {
        // Fermer la fenêtre
        if let Some(window) = window_lock.take() {
            let _ = window.update(cx, |_, window, _| {
                window.remove_window();
            });
        }
    }
}
