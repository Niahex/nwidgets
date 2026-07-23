use std::time::{Duration, SystemTime, UNIX_EPOCH};
use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::corner::{Corner, CornerPosition};
use gpui_component::Icon;
use nwidgets_service_notification::{Notification, NotificationAdded, NotificationService};

const TOAST_TIMEOUT_SECS: u64 = 5;
const MAX_ACTIVE_TOASTS: usize = 5;
const CORNER_RADIUS: f32 = 12.0;

#[derive(Clone)]
pub struct ActiveToast {
    pub notification: Notification,
    pub created_at: u64,
}

pub struct NtfView {
    window_handle: Option<AnyWindowHandle>,
    pub active_toasts: Vec<ActiveToast>,
    _timer_task: Option<Task<()>>,
}

impl NtfView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = NotificationService::init(cx);

        let mut view = Self {
            window_handle: None,
            active_toasts: Vec::new(),
            _timer_task: None,
        };

        cx.subscribe(&service, |this, _, event: &NotificationAdded, cx| {
            this.add_toast(event.notification.clone(), cx);
        })
        .detach();

        view.start_cleanup_timer(cx);
        view
    }

    pub fn set_window_handle(&mut self, handle: AnyWindowHandle) {
        self.window_handle = Some(handle);
    }

    pub fn add_toast(&mut self, notification: Notification, cx: &mut Context<Self>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.active_toasts.insert(
            0,
            ActiveToast {
                notification,
                created_at: now,
            },
        );
        self.active_toasts.truncate(MAX_ACTIVE_TOASTS);

        self.update_window_state(cx);
        cx.notify();
    }

    pub fn dismiss_toast(&mut self, id: u32, cx: &mut Context<Self>) {
        self.active_toasts.retain(|t| t.notification.id != id);
        self.update_window_state(cx);
        cx.notify();
    }

    fn start_cleanup_timer(&mut self, cx: &mut Context<Self>) {
        self._timer_task = Some(cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_secs(1))
                        .await;

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let _ = this.update(&mut cx, |this, cx| {
                        let old_len = this.active_toasts.len();
                        this.active_toasts
                            .retain(|t| now - t.created_at < TOAST_TIMEOUT_SECS);

                        if this.active_toasts.len() != old_len {
                            this.update_window_state(cx);
                            cx.notify();
                        }
                    });
                }
            }
        }));
    }

    fn update_window_state(&self, cx: &mut Context<Self>) {
        if let Some(ref handle) = self.window_handle {
            let handle = handle.clone();
            let count = self.active_toasts.len();
            let visible = count > 0;
            let height = (count as f32 * 110.0).min(550.0) + 16.0 + CORNER_RADIUS;

            let _ = handle.update(cx, |_, window, _| {
                if visible {
                    window.set_layer(gpui::layer_shell::Layer::Overlay);
                    window.set_input_region(None);
                    window.resize(size(px(392.0), px(height)));
                } else {
                    window.set_layer(gpui::layer_shell::Layer::Background);
                    window.set_input_region(None);
                    window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::None);
                    window.resize(size(px(1.0), px(1.0)));
                }
            });
        }
    }
}

impl Render for NtfView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel_bg = rgb(0x2e3440);
        let card_bg = rgb(0x3b4252);
        let frost0 = rgb(0xd8dee9);
        let text_muted = rgb(0xd8dee9);
        let text_bright = rgb(0xe5e9f0);
        let accent = rgb(0x88c0d0);
        let red = rgb(0xbf616a);
        let frost_border = rgb(0x88c0d0).opacity(0.3);

        if self.active_toasts.is_empty() {
            return div().into_any_element();
        }

        let toasts_list = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .w(px(380.0))
            .bg(panel_bg)
            .rounded_bl(px(CORNER_RADIUS))
            .border_l_1()
            .border_r_1()
            .border_b_1()
            .border_color(frost_border)
            .children(self.active_toasts.iter().map(|toast| {
                let notif = &toast.notification;
                let notif_id = notif.id;
                let border_color = match notif.urgency {
                    2 => red,
                    _ => rgb(0x4c566a),
                };

                let body_chars = notif.body.chars().count();
                let truncated_body = if body_chars > 80 {
                    format!("{}…", notif.body.chars().take(80).collect::<String>())
                } else {
                    notif.body.to_string()
                };

                div()
                    .id(SharedString::from(format!("notif-toast-{}", notif.id)))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .bg(card_bg)
                    .rounded_lg()
                    .border_1()
                    .border_color(border_color)
                    .child(
                        // Header
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new("notifications").size(px(16.0)).text_color(accent))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(frost0)
                                            .child(notif.app_name.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_muted)
                                            .child("maintenant"),
                                    )
                                    .child(
                                        div()
                                            .id(SharedString::from(format!("close-toast-{}", notif.id)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(move |this, _, _window, cx| {
                                                this.dismiss_toast(notif_id, cx);
                                            }))
                                            .child(Icon::new("close").size(px(14.0)).text_color(text_muted)),
                                    ),
                            ),
                    )
                    .child(
                        // Summary
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_bright)
                            .child(notif.summary.clone()),
                    )
                    .when(!truncated_body.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(frost0)
                                .child(truncated_body),
                        )
                    })
            }));

        div()
            .size_full()
            .flex()
            .flex_col()
            // ── Top Section: Left concave corner + Main notification body ──
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .flex()
                    .flex_row()
                    // Left column for Top-Left concave corner
                    .child(
                        div()
                            .h_full()
                            .w(px(CORNER_RADIUS))
                            .flex()
                            .flex_col()
                            .child(
                                Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS))
                                    .color(panel_bg)
                                    .border_color(frost_border),
                            )
                            .child(div().flex_1()),
                    )
                    .child(toasts_list),
            )
            // ── Bottom Row: Right concave corner placed BELOW the notification body ──
            .child(
                div()
                    .w_full()
                    .h(px(CORNER_RADIUS))
                    .flex()
                    .flex_row()
                    .child(div().flex_1()) // Transparent space under main body
                    .child(
                        Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS))
                            .color(panel_bg)
                            .border_color(frost_border),
                    ),
            )
            .into_any_element()
    }
}

#[allow(dead_code)]
pub struct NtfManager(pub std::cell::RefCell<Option<AnyWindowHandle>>);

impl NtfManager {
    pub fn new() -> Self {
        Self(std::cell::RefCell::new(None))
    }
}

impl Global for NtfManager {}
