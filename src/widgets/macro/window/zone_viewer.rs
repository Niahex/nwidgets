use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::theme::ActiveTheme;
use crate::widgets::r#macro::types::ClickZone;

static ZONE_VIEWER_WINDOW: OnceLock<Arc<Mutex<Option<WindowHandle<ZoneViewerWidget>>>>> =
    OnceLock::new();

pub struct ZoneViewerWidget {
    focus_handle: FocusHandle,
    zone: ClickZone,
}

impl ZoneViewerWidget {
    pub fn new(cx: &mut Context<Self>, zone: ClickZone) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            zone,
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
}

impl Render for ZoneViewerWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let zone = &self.zone;

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "escape" {
                    close(cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_this, _event: &MouseDownEvent, _window, cx| {
                    close(cx);
                }),
            )
            .w_full()
            .h_full()
            .bg(rgba(0x00000033))
            .child(
                div()
                    .absolute()
                    .left(px(zone.x as f32))
                    .top(px(zone.y as f32))
                    .w(px(zone.width as f32))
                    .h(px(zone.height as f32))
                    .border_2()
                    .border_color(theme.accent)
                    .bg(theme.accent.opacity(0.2))
                    .child(
                        div()
                            .absolute()
                            .top(px(-25.0))
                            .left(px(0.0))
                            .px_2()
                            .py_1()
                            .bg(theme.accent)
                            .rounded(px(4.))
                            .text_xs()
                            .text_color(theme.bg)
                            .child(format!("{}x{}", zone.width, zone.height)),
                    ),
            )
    }
}

pub fn open(cx: &mut App, zone: ClickZone) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let display = cx.displays().first().cloned();
    let bounds = display.map(|d| d.bounds()).unwrap_or_else(|| Bounds {
        origin: Point::default(),
        size: Size {
            width: px(1920.0),
            height: px(1080.0),
        },
    });

    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                window_decorations: Some(WindowDecorations::Client),
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-zone-viewer".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::all(),
                    exclusive_zone: None,
                    margin: None,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                app_id: Some("nwidgets-zone-viewer".to_string()),
                is_movable: false,
                ..Default::default()
            },
            move |_window, cx| cx.new(|cx| ZoneViewerWidget::new(cx, zone)),
        )
        .expect("Failed to create zone viewer window");

    window
        .update(cx, |widget, window, cx| {
            window.focus(widget.focus_handle(), cx);
            cx.activate(true);
        })
        .ok();

    ZONE_VIEWER_WINDOW
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .lock()
        .replace(window);
}

pub fn close(_cx: &mut App) {
    if let Some(window_arc) = ZONE_VIEWER_WINDOW.get() {
        let mut lock = window_arc.lock();
        *lock = None;
    }
}
