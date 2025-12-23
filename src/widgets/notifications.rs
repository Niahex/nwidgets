use crate::services::notifications::{
    Notification, NotificationAdded, NotificationService,
};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct NotificationsStateChanged {
    pub has_notifications: bool,
}

pub struct NotificationsWidget {
    service: Entity<NotificationService>,
    notifications: Arc<RwLock<Vec<Notification>>>,
}

impl EventEmitter<NotificationsStateChanged> for NotificationsWidget {}

impl NotificationsWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = NotificationService::global(cx);
        let notifications = Arc::new(RwLock::new(Vec::new()));

        // S'abonner aux nouvelles notifications
        let notifications_clone = Arc::clone(&notifications);
        cx.subscribe(
            &service,
            move |_this, _service, event: &NotificationAdded, cx| {
                let mut notifs = notifications_clone.write();

                // Ajouter en début de liste (plus récent en haut)
                notifs.insert(0, event.notification.clone());
                notifs.truncate(10); // Max 10 notifications

                drop(notifs);
                cx.emit(NotificationsStateChanged {
                    has_notifications: true,
                });
                cx.notify();
            },
        )
        .detach();

        // Timer pour nettoyer les notifications expirées
        let notifications_clone = Arc::clone(&notifications);
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut notifs = notifications_clone.write();
            let old_count = notifs.len();
            notifs.retain(|n| now - n.timestamp < 5);

            if notifs.len() != old_count {
                let is_empty = notifs.is_empty();
                drop(notifs);
                let _ = this.update(cx, |_, cx| {
                    if is_empty {
                        cx.emit(NotificationsStateChanged {
                            has_notifications: false,
                        });
                    }
                    cx.notify();
                });
            }
        })
        .detach();

        Self {
            service,
            notifications,
        }
    }
}

impl Render for NotificationsWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifs = self.notifications.read().clone();

        let theme = cx.global::<crate::theme::Theme>();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(380.0))
            .children(notifs.iter().map(|notif| {
                let urgency_class = match notif.urgency {
                    2 => theme.error,
                    1 => theme.bg,
                    _ => theme.hover,
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .bg(urgency_class)
                    .rounded(px(12.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(if !notif.app_icon.is_empty() {
                                        Icon::new(&notif.app_icon)
                                            .size(px(20.0))
                                            .color(theme.text)
                                            .preserve_colors(true)
                                            .into_any_element()
                                    } else {
                                        div().size_4().into_any_element()
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text)
                                            .child(notif.app_name.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format_time_ago(notif.timestamp)),
                            ),
                    )
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text)
                            .child(notif.summary.clone()),
                    )
                    .when(!notif.body.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(theme.text_bright)
                                .child(notif.body.clone()),
                        )
                    })
            }))
    }
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

pub struct NotificationsWindowManager {
    window: Option<WindowHandle<NotificationsWidget>>,
}

impl NotificationsWindowManager {
    pub fn new() -> Self {
        Self { window: None }
    }

    pub fn open_window(&mut self, cx: &mut App) -> Option<Entity<NotificationsWidget>> {
        if self.window.is_some() {
            return self
                .window
                .as_ref()
                .and_then(|w| cx.read_window(w, |entity, _| entity.clone()).ok());
        }

        use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px(3040.0),
                            y: px(60.0),
                        },
                        size: Size {
                            width: px(400.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-notifications".to_string(),
                        layer: Layer::Overlay,
                        anchor: Anchor::TOP | Anchor::RIGHT,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(NotificationsWidget::new),
            )
            .ok()?;

        let entity = cx.read_window(&window, |entity, _| entity.clone()).ok();
        self.window = Some(window);
        entity
    }

    pub fn close_window(&mut self, cx: &mut App) {
        if let Some(window) = self.window.take() {
            window
                .update(cx, |_, window, _| {
                    window.remove_window();
                })
                .ok();
        }
    }
}
