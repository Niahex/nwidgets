use crate::widgets::notifications::service::state::NotificationState;
use crate::widgets::notifications::types::{Notification, HISTORY_CAPACITY};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;

pub struct NotificationServer {
    next_id: u32,
    state: Arc<Mutex<NotificationState>>,
}

impl NotificationServer {
    pub fn new(state: Arc<Mutex<NotificationState>>) -> Self {
        Self { next_id: 0, state }
    }
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::Value>,
        _expire_timeout: i32,
    ) -> u32 {
        log::debug!("Notification received - app: '{app_name}', summary: '{summary}'");

        if app_name.to_lowercase() == "spotify" {
            log::debug!("Ignoring Spotify notification");
            let id = if replaces_id > 0 {
                replaces_id
            } else {
                self.next_id += 1;
                self.next_id
            };
            return id;
        }

        let id = if replaces_id > 0 {
            replaces_id
        } else {
            self.next_id += 1;
            self.next_id
        };

        let urgency = hints
            .get("urgency")
            .and_then(|v| v.downcast_ref::<u8>().ok())
            .unwrap_or(1);

        let notification = Notification {
            app_name: app_name.into(),
            summary: summary.into(),
            body: body.into(),
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            actions,
            app_icon: app_icon.into(),
        };

        let mut state = self.state.lock();
        state.history.push_front(notification.clone());
        if state.history.len() > HISTORY_CAPACITY {
            state.history.pop_back();
        }

        if let Some(sender) = &state.sender {
            if let Err(e) = sender.send(notification) {
                log::error!("Failed to send notification to UI: {e}");
            }
        }

        id
    }

    fn close_notification(&mut self, _id: u32) {}

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

pub async fn run_dbus_server(
    state: Arc<Mutex<NotificationState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::session().await?;
    let server = NotificationServer::new(state);

    connection
        .object_server()
        .at("/org/freedesktop/Notifications", server)
        .await?;

    connection
        .request_name("org.freedesktop.Notifications")
        .await?;

    log::info!("Notification service ready on org.freedesktop.Notifications");
    std::future::pending::<()>().await;
    Ok(())
}
