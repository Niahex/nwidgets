use crate::theme::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::{
    div, point, prelude::*, px, rgb, App, Bounds, Context, Entity, IntoElement, Render, Size,
    Window, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use zbus::Connection;

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: u8,
    pub timestamp: u64,
}

pub struct NotificationService {
    notifications: Arc<Mutex<Vec<Notification>>>,
    sender: mpsc::UnboundedSender<Notification>,
}

struct NotificationServer {
    notifications: Arc<Mutex<Vec<Notification>>>,
    sender: mpsc::UnboundedSender<Notification>,
    next_id: u32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::Value>,
        _expire_timeout: i32,
    ) -> u32 {
        println!(
            "[NOTIF] 📨 Received notification - app: '{}', summary: '{}', body: '{}'",
            app_name, summary, body
        );

        let id = if replaces_id > 0 {
            println!("[NOTIF] Replacing notification ID: {}", replaces_id);
            replaces_id
        } else {
            self.next_id += 1;
            println!("[NOTIF] New notification ID: {}", self.next_id);
            self.next_id
        };

        let urgency = if let Some(value) = hints.get("urgency") {
            if let Ok(u) = value.downcast_ref::<u8>() {
                println!("[NOTIF] Urgency from hints: {}", u);
                u.clone()
            } else {
                println!("[NOTIF] Failed to parse urgency, using default: 1");
                1
            }
        } else {
            println!("[NOTIF] No urgency hint, using default: 1");
            1
        };

        let notification = Notification {
            id,
            app_name: app_name.clone(),
            summary: summary.clone(),
            body: body.clone(),
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        {
            let mut notifications = self.notifications.lock().unwrap();
            if let Some(pos) = notifications.iter().position(|n| n.id == id) {
                println!("[NOTIF] Updating existing notification at position {}", pos);
                notifications[pos] = notification.clone();
            } else {
                println!("[NOTIF] Adding new notification");
                notifications.push(notification.clone());
            }
            notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            notifications.truncate(10);
            println!(
                "[NOTIF] Total notifications in storage: {}",
                notifications.len()
            );
        }

        match self.sender.send(notification.clone()) {
            Ok(_) => println!("[NOTIF] ✅ Notification sent to channel successfully"),
            Err(e) => println!("[NOTIF] ❌ Failed to send notification to channel: {}", e),
        }

        id
    }

    fn close_notification(&mut self, id: u32) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.retain(|n| n.id != id);
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "body-markup".to_string(),
            "actions".to_string(),
            "urgency".to_string(),
        ]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "nwidgets".to_string(),
            "nwidgets".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

impl NotificationService {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Notification>) {
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let (sender, receiver) = mpsc::unbounded_channel();

        let service = Self {
            notifications: notifications.clone(),
            sender: sender.clone(),
        };

        (service, receiver)
    }

    pub fn start_dbus_server(&self) {
        let notifications = self.notifications.clone();
        let sender = self.sender.clone();

        println!("[NOTIF] 🚀 Starting D-Bus server thread");

        std::thread::spawn(move || {
            println!("[NOTIF] 🔧 D-Bus thread started, creating runtime");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                println!("[NOTIF] 🔧 Running D-Bus server");
                if let Err(e) = Self::run_dbus_server(notifications, sender).await {
                    eprintln!("[NOTIF] ❌ Erreur D-Bus: {}", e);
                } else {
                    println!("[NOTIF] ✅ D-Bus server running");
                }
            });
        });
    }

    async fn run_dbus_server(
        notifications: Arc<Mutex<Vec<Notification>>>,
        sender: mpsc::UnboundedSender<Notification>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("[NOTIF] 🔌 Connecting to D-Bus session bus");
        let connection = Connection::session().await?;
        println!("[NOTIF] ✅ Connected to D-Bus session bus");

        let server = NotificationServer {
            notifications,
            sender,
            next_id: 0,
        };

        println!("[NOTIF] 📍 Registering object at /org/freedesktop/Notifications");
        connection
            .object_server()
            .at("/org/freedesktop/Notifications", server)
            .await?;
        println!("[NOTIF] ✅ Object registered");

        println!("[NOTIF] 🏷️  Requesting name org.freedesktop.Notifications");
        connection
            .request_name("org.freedesktop.Notifications")
            .await?;
        println!("[NOTIF] ✅ Name acquired: org.freedesktop.Notifications");
        println!("[NOTIF] 🎉 D-Bus server is now ready to receive notifications!");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

// NotificationManager - gère l'affichage des notifications dans des fenêtres
pub struct NotificationManager {
    notifications: Vec<Notification>,
    notification_windows: Vec<(u64, WindowHandle<NotificationWindow>)>, // (timestamp, window)
}

struct NotificationWindow {
    notifications: Vec<Notification>,
}

impl NotificationManager {
    pub fn new(cx: &mut App) -> Entity<Self> {
        let (service, mut receiver) = NotificationService::new();
        service.start_dbus_server();

        let manager = cx.new(|_cx| Self {
            notifications: Vec::new(),
            notification_windows: Vec::new(),
        });

        // Spawn task pour recevoir les notifications
        cx.spawn({
            let manager = manager.clone();
            async move |cx| {
                println!("[NOTIF_MANAGER] 📢 Notification receiver task started");
                while let Some(notification) = receiver.recv().await {
                    println!(
                        "[NOTIF_MANAGER] 📢 Received notification: {} - {}",
                        notification.summary, notification.body
                    );

                    let _ = manager.update(cx, |this, cx| {
                        // Ajouter la notification
                        this.notifications.push(notification.clone());
                        this.notifications
                            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        this.notifications.truncate(10);

                        println!(
                            "[NOTIF_MANAGER] 📢 Now have {} notifications",
                            this.notifications.len()
                        );

                        // Créer ou mettre à jour la fenêtre
                        this.update_window(cx);
                    });
                }
                println!("[NOTIF_MANAGER] ⚠️  Notification receiver channel closed!");
            }
        })
        .detach();

        // Spawn task pour nettoyer les notifications expirées
        cx.spawn({
            let manager = manager.clone();
            async move |cx| {
                println!("[NOTIF_MANAGER] 📢 Notification cleanup task started");
                loop {
                    cx.background_executor().timer(Duration::from_secs(1)).await;

                    let _ = manager.update(cx, |this, cx| {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let old_count = this.notifications.len();
                        this.notifications.retain(|n| now - n.timestamp < 5);

                        if this.notifications.len() != old_count {
                            println!(
                                "[NOTIF_MANAGER] 📢 Cleaned up notifications: {} -> {}",
                                old_count,
                                this.notifications.len()
                            );
                            this.update_window(cx);
                        }
                    });
                }
            }
        })
        .detach();

        manager
    }

    fn update_window(&mut self, cx: &mut Context<Self>) {
        println!(
            "[NOTIF_MANAGER] 🔄 Updating windows, current count: {}, notifications: {}",
            self.notification_windows.len(),
            self.notifications.len()
        );

        // Supprimer les fenêtres des notifications qui n'existent plus
        let mut to_remove = Vec::new();
        for (i, (timestamp, window)) in self.notification_windows.iter().enumerate() {
            let exists = self.notifications.iter().any(|n| n.timestamp == *timestamp);
            if !exists {
                println!(
                    "[NOTIF_MANAGER] 🗑️  Marking window {} for removal (notification {})",
                    i, timestamp
                );
                to_remove.push(i);

                // Fermer explicitement la fenêtre
                let _ = window.update(cx, |_, window, _| {
                    window.remove_window();
                });
            }
        }

        // Retirer les fenêtres marquées pour suppression (en ordre inverse pour garder les indices valides)
        for i in to_remove.iter().rev() {
            self.notification_windows.remove(*i);
            println!("[NOTIF_MANAGER] ✅ Window removed");
        }

        // Si le nombre de fenêtres a changé, on doit recréer toutes les fenêtres
        // pour repositionner correctement (car les marges dépendent de l'index)
        if self.notification_windows.len() != self.notifications.len() {
            println!("[NOTIF_MANAGER] 📊 Window count changed, recreating all");
            self.recreate_all_windows(cx);
        }
    }

    fn recreate_all_windows(&mut self, cx: &mut Context<Self>) {
        println!("[NOTIF_MANAGER] 🔄 Recreating all windows to reposition");

        // Fermer explicitement toutes les fenêtres existantes
        let old_count = self.notification_windows.len();
        for (timestamp, window) in &self.notification_windows {
            println!(
                "[NOTIF_MANAGER] 🗑️  Closing window for notification {}",
                timestamp
            );
            let _ = window.update(cx, |_, window, _| {
                window.remove_window();
            });
        }
        self.notification_windows.clear();
        println!(
            "[NOTIF_MANAGER] ✅ Closed and cleared {} old windows",
            old_count
        );

        // Si pas de notifications, on ne crée rien
        if self.notifications.is_empty() {
            println!("[NOTIF_MANAGER] ✅ No notifications, no windows to create");
            return;
        }

        // Recréer les fenêtres aux bonnes positions (du haut vers le bas)
        // Avec LayerShell et anchor TOP|RIGHT, les marges sont:
        // - top: distance depuis le haut de l'écran
        // - right: distance depuis le bord droit
        const NOTIF_WIDTH: f32 = 380.0;
        const NOTIF_HEIGHT: f32 = 100.0;
        const MARGIN_RIGHT: f32 = 20.0;
        const MARGIN_TOP_BASE: f32 = 68.0; // 48px panel + 20px marge
        const SPACING: f32 = 10.0;

        for (index, notification) in self.notifications.iter().enumerate() {
            // Calculer la marge top pour cette notification
            let margin_top = MARGIN_TOP_BASE + (index as f32 * (NOTIF_HEIGHT + SPACING));

            println!(
                "[NOTIF_MANAGER] 🪟 Creating window {} with margin_top={} for: {}",
                index, margin_top, notification.summary
            );

            let notif_clone = notification.clone();
            match cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(0.), px(0.)), // Ignoré avec anchor
                        size: Size::new(px(NOTIF_WIDTH), px(NOTIF_HEIGHT)),
                    })),
                    app_id: Some(format!("nwidgets-notification-{}", notification.timestamp)),
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: format!("nwidgets-notification-{}", notification.timestamp),
                        layer: Layer::Overlay,
                        anchor: Anchor::TOP | Anchor::RIGHT, // Ancré en haut à droite
                        margin: Some((px(margin_top), px(MARGIN_RIGHT), px(0.0), px(0.0))), // top, right, bottom, left
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_cx| NotificationWindow {
                        notifications: vec![notif_clone],
                    })
                },
            ) {
                Ok(window) => {
                    self.notification_windows
                        .push((notification.timestamp, window));
                    println!("[NOTIF_MANAGER] ✅ Window created successfully");
                }
                Err(e) => {
                    println!("[NOTIF_MANAGER] ❌ Failed to recreate window: {:?}", e);
                }
            }
        }

        println!(
            "[NOTIF_MANAGER] ✅ Recreated {} windows",
            self.notification_windows.len()
        );
    }
}

impl Render for NotificationWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = div().size_full().flex().flex_col().gap_3().p_4();

        for notification in &self.notifications {
            let border_color = match notification.urgency {
                2 => RED,    // Critical - rouge
                1 => YELLOW, // Normal - jaune
                _ => FROST1, // Low - bleu
            };

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - notification.timestamp;

            let time_str = if elapsed < 60 {
                "now".to_string()
            } else if elapsed < 3600 {
                format!("{}m ago", elapsed / 60)
            } else {
                format!("{}h ago", elapsed / 3600)
            };

            let mut notif_div = div()
                .w_full()
                .bg(rgb(POLAR2))
                .rounded_lg()
                .p_4()
                .border_l_4()
                .border_color(rgb(border_color))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_color(rgb(SNOW0))
                                .text_sm()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(notification.summary.clone()),
                        )
                        .child(div().text_color(rgb(POLAR3)).text_xs().child(time_str)),
                )
                .child(
                    div()
                        .text_color(rgb(SNOW0))
                        .text_xs()
                        .child(notification.body.clone()),
                );

            if !notification.app_name.is_empty() {
                notif_div = notif_div.child(
                    div()
                        .text_color(rgb(POLAR3))
                        .text_xs()
                        .child(format!("from {}", notification.app_name)),
                );
            }

            container = container.child(notif_div);
        }

        container
    }
}
