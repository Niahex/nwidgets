use parking_lot::RwLock;
use std::sync::Arc;
use zbus::{connection, interface, Connection};
use crate::TOKIO_RUNTIME;

pub struct DbusLauncherService {
    callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl DbusLauncherService {
    pub fn new() -> Self {
        let service = Self {
            callback: Arc::new(RwLock::new(None)),
        };
        
        let callback = service.callback.clone();
        
        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::start_dbus_server(callback).await {
                log::error!("Failed to start D-Bus launcher service: {}", e);
            }
        });
        
        service
    }
    
    async fn start_dbus_server(callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>) -> anyhow::Result<()> {
        let conn = connection::Builder::session()?
            .name("org.nwidgets.Launcher")?
            .build()
            .await?;

        let interface = LauncherInterface { callback };
        
        conn.object_server()
            .at("/org/nwidgets/Launcher", interface)
            .await?;

        log::info!("D-Bus launcher service started at org.nwidgets.Launcher");

        // Keep the connection alive
        std::future::pending::<()>().await;
        
        Ok(())
    }
    
    pub fn on_toggle<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self.callback.write() = Some(Box::new(callback));
    }
}

struct LauncherInterface {
    callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

#[interface(name = "org.nwidgets.Launcher")]
impl LauncherInterface {
    fn toggle_launcher(&self) {
        log::info!("Toggle launcher requested via D-Bus");
        if let Some(cb) = self.callback.read().as_ref() {
            cb();
        }
    }
}
