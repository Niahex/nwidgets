use crate::services::notifications::{Notification, NotificationService, NotificationAdded};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions, Layer};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct NotificationsWidget {
    service: Entity<NotificationService>,
    notifications: Arc<RwLock<Vec<Notification>>>,
}

impl NotificationsWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = NotificationService::global(cx);
        let notifications = Arc::new(RwLock::new(Vec::new()));

        // S'abonner aux nouvelles notifications
        let notifications_clone: Arc<RwLock<Vec<Notification>>> = Arc::clone(&notifications);
        cx.subscribe(&service, move |this, _service, event: &NotificationAdded, cx| {
            println!("[NOTIF_WIDGET] 📬 New notification received: {}", event.notification.summary);

            let mut notifs = notifications_clone.write();
            // Ajouter seulement si pas déjà présent
            if !notifs.iter().any(|n: &Notification| n.timestamp == event.notification.timestamp && n.summary == event.notification.summary) {
                notifs.push(event.notification.clone());
                // Trier par timestamp décroissant
                notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                // Garder max 10 notifications
                notifs.truncate(10);
            }
            drop(notifs);

            cx.notify();
        })
        .detach();

        // Timer pour nettoyer les notifications expirées (toutes les secondes)
        let notifications_clone: Arc<RwLock<Vec<Notification>>> = Arc::clone(&notifications);
        cx.spawn(async move |this, mut cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(1))
                    .await;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut notifs = notifications_clone.write();
                let old_count = notifs.len();
                notifs.retain(|n: &Notification| now - n.timestamp < 5);

                if notifs.len() != old_count {
                    println!(
                        "[NOTIF_GPUI] 🗑️  Cleaned up notifications: {} -> {}",
                        old_count,
                        notifs.len()
                    );
                    drop(notifs);

                    let _ = this.update(cx, |_, cx| {
                        cx.notify();
                    });
                }
            }
        })
        .detach();

        Self {
            service,
            notifications,
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

    pub fn has_notifications(&self) -> bool {
        !self.notifications.read().is_empty()
    }
}

impl Render for NotificationsWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifs = self.notifications.read().clone();

        // Si pas de notifications, rendre un div vide
        if notifs.is_empty() {
            return div().into_any_element();
        }

        // Nord colors
        let bg_color = rgb(0x2e3440); // polar0
        let text_color = rgb(0xeceff4); // snow3
        let time_color = rgb(0xd8dee9); // snow1
        let body_color = rgb(0xe5e9f0); // snow2

        // Container principal
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(380.0))
            .children(notifs.iter().map(|notif| {
                let urgency_class = match notif.urgency {
                    2 => rgb(0xbf616a), // critical - red
                    1 => bg_color,       // normal - default
                    _ => rgb(0x4c566a),  // low - darker
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .bg(urgency_class)
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
            }))
            .into_any_element()
    }
}

// Gestionnaire global pour ouvrir/fermer la fenêtre de notifications
pub struct NotificationsWindowManager {
    window_handle: Option<AnyWindowHandle>,
}

impl NotificationsWindowManager {
    pub fn new() -> Self {
        Self {
            window_handle: None,
        }
    }

    pub fn open_window(&mut self, cx: &mut App) {
        if self.window_handle.is_some() {
            return; // Déjà ouverte
        }

        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(3440.0 - 380.0 - 10.0),
                        y: px(10.0),
                    },
                    size: Size {
                        width: px(380.0),
                        height: px(1000.0),
                    },
                })),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-notifications".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP | Anchor::RIGHT,
                    exclusive_zone: None,
                    margin: Some((px(10.0), px(10.0), px(0.0), px(0.0))),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| NotificationsWidget::new(cx)),
        );

        if let Ok(handle) = handle {
            self.window_handle = Some(handle.into());
        }
    }

    pub fn close_window(&mut self, cx: &mut App) {
        if let Some(handle) = self.window_handle.take() {
            let _ = handle.update(cx, |_, window, _| {
                window.remove_window();
            });
        }
    }
}
