use parking_lot::RwLock;
use std::sync::Arc;
use zbus::{connection, interface};
use crate::TOKIO_RUNTIME;

pub struct DbusTaskerService {
    callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl DbusTaskerService {
    pub fn new() -> Self {
        let service = Self {
            callback: Arc::new(RwLock::new(None)),
        };
        
        let callback = service.callback.clone();
        
        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::start_dbus_server(callback).await {
                log::error!("Failed to start D-Bus tasker service: {}", e);
            }
        });
        
        service
    }
    
    async fn start_dbus_server(callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>) -> anyhow::Result<()> {
        let conn = connection::Builder::session()?
            .name("org.nwidgets.Tasker")?
            .build()
            .await?;

        let interface = TaskerInterface { callback };
        
        conn.object_server()
            .at("/org/nwidgets/Tasker", interface)
            .await?;

        log::info!("D-Bus tasker service started at org.nwidgets.Tasker");

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

struct TaskerInterface {
    callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

#[interface(name = "org.nwidgets.Tasker")]
impl TaskerInterface {
    fn toggle_tasker(&self) {
        log::info!("Toggle tasker requested via D-Bus");
        if let Some(cb) = self.callback.read().as_ref() {
            cb();
        }
    }
}
