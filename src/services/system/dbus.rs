use zbus::{connection, interface, Connection};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::TOKIO_RUNTIME;

pub struct DbusService {
    connection: Arc<RwLock<Option<Connection>>>,
}

impl DbusService {
    pub fn new() -> Self {
        let service = Self {
            connection: Arc::new(RwLock::new(None)),
        };

        service.start();
        service
    }

    fn start(&self) {
        let connection = self.connection.clone();

        TOKIO_RUNTIME.spawn(async move {
            match Self::setup_dbus().await {
                Ok(conn) => {
                    *connection.write() = Some(conn);
                    log::info!("D-Bus service started");
                }
                Err(e) => {
                    log::error!("Failed to start D-Bus service: {}", e);
                }
            }
        });
    }

    async fn setup_dbus() -> anyhow::Result<Connection> {
        let conn = connection::Builder::session()?
            .name("org.nwidgets.Daemon")?
            .build()
            .await?;

        Ok(conn)
    }
}

pub struct NWidgetsInterface;

#[interface(name = "org.nwidgets.Daemon")]
impl NWidgetsInterface {
    async fn toggle_launcher(&self) -> bool {
        log::info!("Toggle launcher requested via D-Bus");
        true
    }

    async fn toggle_control_center(&self) -> bool {
        log::info!("Toggle control center requested via D-Bus");
        true
    }

    async fn show_notification(&self, summary: &str, body: &str) -> u32 {
        log::info!("Show notification: {} - {}", summary, body);
        0
    }
}
