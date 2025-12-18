use crate::services::notifications::Notification;
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions, Layer};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub struct NotificationWidget {
    notification: Notification,
    window_handle: AnyWindowHandle,
}

impl NotificationWidget {
    pub fn new(notification: Notification, window_handle: AnyWindowHandle, cx: &mut Context<Self>) -> Self {
        // Auto-fermer après 5 secondes
        let handle = window_handle.clone();
        cx.spawn(async move |_this, mut cx| {
            cx.background_executor()
                .timer(Duration::from_secs(5))
                .await;

            let _ = cx.update(|cx| {
                let _ = handle.update(cx, |_, window, _| {
                    window.remove_window();
                });
            });
        })
        .detach();

        Self { notification, window_handle }
    }

    fn format_time_ago(timestamp: u64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed = now.saturating_sub(timestamp);

        if elapsed < 60 {
            "now".to_string()
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else {
            format!("{}h ago", elapsed / 3600)
        }
    }
}

impl Render for NotificationWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notif = &self.notification;

        // Nord colors
        let bg_color = match notif.urgency {
            2 => rgb(0xbf616a), // critical - red
            1 => rgb(0x2e3440),  // normal - default
            _ => rgb(0x4c566a),  // low - darker
        };
        let text_color = rgb(0xeceff4); // snow3
        let time_color = rgb(0xd8dee9); // snow1
        let body_color = rgb(0xe5e9f0); // snow2

        div()
            .w(px(380.0))
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(bg_color)
            .rounded(px(12.0))
            .child(
                // Header: icône app + nom app + heure
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(
                                // Icône de l'application
                                if !notif.app_icon.is_empty() {
                                    Icon::new(&notif.app_icon)
                                        .size(px(20.0))
                                        .color(text_color)
                                        .preserve_colors(true)
                                        .into_any_element()
                                } else {
                                    div().size_4().into_any_element()
                                }
                            )
                            .child(
                                // Nom de l'application
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text_color)
                                    .child(notif.app_name.clone())
                            )
                    )
                    .child(
                        // Heure
                        div()
                            .text_xs()
                            .text_color(time_color)
                            .child(Self::format_time_ago(notif.timestamp))
                    )
            )
            .child(
                // Summary (titre)
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_color)
                    .child(notif.summary.clone())
            )
            .when(!notif.body.is_empty(), |this| {
                this.child(
                    // Body (contenu)
                    div()
                        .text_sm()
                        .text_color(body_color)
                        .child(notif.body.clone())
                )
            })
            .when(!notif.actions.is_empty(), |this| {
                // Actions (boutons)
                this.child(
                    div()
                        .flex()
                        .gap_2()
                        .mt_2()
                        .children(
                            notif.actions.chunks(2).filter_map(|chunk| {
                                if chunk.len() == 2 {
                                    let label = &chunk[1];
                                    Some(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .bg(rgb(0x4c566a))
                                            .rounded(px(6.0))
                                            .text_sm()
                                            .text_color(text_color)
                                            .child(label.clone())
                                    )
                                } else {
                                    None
                                }
                            })
                        )
                )
            })
    }
}

// Gestionnaire pour créer une fenêtre par notification
pub fn create_notification_window(notification: Notification, index: usize, cx: &mut App) {
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point {
                    x: px(3440.0 - 380.0 - 10.0),
                    y: px(10.0 + (index as f32 * 110.0)), // Espacer les notifications
                },
                size: Size {
                    width: px(380.0),
                    height: px(100.0),
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: format!("nwidgets-notification-{}", notification.timestamp),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::RIGHT,
                exclusive_zone: None,
                margin: Some((px(10.0 + (index as f32 * 110.0)), px(10.0), px(0.0), px(0.0))),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            let window_handle: AnyWindowHandle = window.window_handle().into();
            cx.new(|cx| NotificationWidget::new(notification, window_handle, cx))
        },
    );
}
