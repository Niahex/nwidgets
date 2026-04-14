use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::theme::ActiveTheme;
use crate::widgets::r#macro::types::ClickZone;

static ZONE_SELECTOR_WINDOW: OnceLock<Arc<Mutex<WindowHandle<ZoneSelectorWidget>>>> =
    OnceLock::new();

pub struct ZoneSelectorWidget {
    focus_handle: FocusHandle,
    start_pos: Option<Point<Pixels>>,
    current_pos: Option<Point<Pixels>>,
    callback: Arc<Mutex<Option<Box<dyn Fn(ClickZone) + Send + 'static>>>>,
}

impl ZoneSelectorWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            start_pos: None,
            current_pos: None,
            callback: Arc::new(Mutex::new(None)),
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn get_zone(&self) -> Option<ClickZone> {
        let start = self.start_pos?;
        let current = self.current_pos?;

        let x: f32 = start.x.min(current.x).into();
        let y: f32 = start.y.min(current.y).into();
        let width: f32 = (start.x.max(current.x) - start.x.min(current.x)).into();
        let height: f32 = (start.y.max(current.y) - start.y.min(current.y)).into();

        Some(ClickZone {
            x: x as i32,
            y: y as i32,
            width: width as u32,
            height: height as u32,
        })
    }
}

impl Render for ZoneSelectorWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let start = self.start_pos;
        let current = self.current_pos;

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "escape" {
                    close(cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    this.start_pos = Some(event.position);
                    this.current_pos = Some(event.position);
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if this.start_pos.is_some() {
                    this.current_pos = Some(event.position);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    if let Some(zone) = this.get_zone() {
                        if let Some(callback) = this.callback.lock().take() {
                            callback(zone);
                        }
                    }
                    close(cx);
                }),
            )
            .w_full()
            .h_full()
            .bg(rgba(0x00000066))
            .children(if let (Some(start), Some(current)) = (start, current) {
                let x = start.x.min(current.x);
                let y = start.y.min(current.y);
                let width = start.x.max(current.x) - start.x.min(current.x);
                let height = start.y.max(current.y) - start.y.min(current.y);

                Some(
                    div()
                        .absolute()
                        .left(x)
                        .top(y)
                        .w(width)
                        .h(height)
                        .border_2()
                        .border_color(theme.accent)
                        .bg(theme.accent.opacity(0.2)),
                )
            } else {
                None
            })
    }
}

pub fn open<F>(cx: &mut App, callback: F)
where
    F: Fn(ClickZone) + Send + 'static,
{
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
                    namespace: "nwidgets-zone-selector".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::all(),
                    exclusive_zone: None,
                    margin: None,
                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                    ..Default::default()
                }),
                app_id: Some("nwidgets-zone-selector".to_string()),
                is_movable: false,
                ..Default::default()
            },
            move |_window, cx| {
                let widget = cx.new(|cx| ZoneSelectorWidget::new(cx));
                widget.update(cx, |w, _cx| {
                    *w.callback.lock() = Some(Box::new(callback));
                });
                widget
            },
        )
        .expect("Failed to create zone selector window");

    window
        .update(cx, |widget, window, cx| {
            window.focus(widget.focus_handle(), cx);
            cx.activate(true);
        })
        .ok();

    ZONE_SELECTOR_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn close(_cx: &mut App) {
    if let Some(window_arc) = ZONE_SELECTOR_WINDOW.get() {
        let _window = window_arc.lock().clone();
    }
}
