use std::sync::Arc;
use parking_lot::RwLock;
use zbus::{connection, interface, Connection};

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug)]
pub struct NotificationData {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub timeout: i32,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct NotificationService {
    state: Arc<RwLock<NotificationState>>,
}

#[derive(Default)]
struct NotificationState {
    notifications: Vec<NotificationData>,
    next_id: u32,
}

impl NotificationService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(NotificationState {
                next_id: 1,
                ..Default::default()
            })),
        };

        service.start();
        service
    }

    fn start(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::start_server(state).await {
                log::error!("Notification server error: {}", e);
            }
        });
    }

    async fn start_server(_state: Arc<RwLock<NotificationState>>) -> anyhow::Result<()> {
        log::info!("Notification service started");
        Ok(())
    }

    pub fn get_notifications(&self) -> Vec<NotificationData> {
        self.state.read().notifications.clone()
    }

    pub fn dismiss(&self, id: u32) {
        self.state.write().notifications.retain(|n| n.id != id);
    }

    pub fn dismiss_all(&self) {
        self.state.write().notifications.clear();
    }
}
