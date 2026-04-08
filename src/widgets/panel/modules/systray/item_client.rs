use super::types::{TrayCategory, TrayIcon, TrayItem, TrayStatus};
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.kde.StatusNotifierItem",
    default_service = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem"
)]
trait StatusNotifierItem {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn category(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;

    #[zbus(property)]
    fn attention_icon_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn menu(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;

    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;

    fn scroll(&self, delta: i32, orientation: &str) -> zbus::Result<()>;
}

pub async fn fetch_item_data(service: &str, object_path: &str) -> anyhow::Result<TrayItem> {
    // Add timeout to prevent indefinite hangs
    let timeout = std::time::Duration::from_secs(2);
    
    let result = tokio::time::timeout(timeout, async {
        let connection = Connection::session().await?;
        
        let proxy = StatusNotifierItemProxy::builder(&connection)
            .destination(service)?
            .path(object_path)?
            .build()
            .await?;

        let id = proxy.id().await.unwrap_or_else(|_| object_path.to_string());
        let title = proxy.title().await.unwrap_or_default();
        let status_str = proxy.status().await.unwrap_or_else(|_| "Active".to_string());
        let category_str = proxy.category().await.unwrap_or_else(|_| "ApplicationStatus".to_string());
        let icon_name = proxy.icon_name().await.ok().filter(|s| !s.is_empty());
        let icon_pixmap_raw = proxy.icon_pixmap().await.ok();
        let attention_icon_name = proxy.attention_icon_name().await.ok().filter(|s| !s.is_empty());
        let menu_path = proxy.menu().await.ok().map(|p| p.to_string());

        let status = match status_str.as_str() {
            "Passive" => TrayStatus::Passive,
            "Active" => TrayStatus::Active,
            "NeedsAttention" => TrayStatus::NeedsAttention,
            _ => TrayStatus::Active,
        };

        let category = match category_str.as_str() {
            "ApplicationStatus" => TrayCategory::ApplicationStatus,
            "Communications" => TrayCategory::Communications,
            "SystemServices" => TrayCategory::SystemServices,
            "Hardware" => TrayCategory::Hardware,
            _ => TrayCategory::ApplicationStatus,
        };

        let icon_pixmap = icon_pixmap_raw.map(|pixmaps| {
            pixmaps
                .into_iter()
                .map(|(width, height, data)| TrayIcon { width, height, data })
                .collect()
        });

        Ok::<TrayItem, anyhow::Error>(TrayItem {
            service: service.to_string(),
            object_path: object_path.to_string(),
            id,
            title,
            status,
            category,
            icon_name,
            icon_pixmap,
            attention_icon_name,
            menu_path,
        })
    })
    .await;
    
    match result {
        Ok(item) => item,
        Err(_) => Err(anyhow::anyhow!("Timeout fetching tray item data for {}", service)),
    }
}

pub async fn activate_item(service: &str, object_path: &str, x: i32, y: i32) -> anyhow::Result<()> {
    let timeout = std::time::Duration::from_secs(2);
    
    let result = tokio::time::timeout(timeout, async {
        let connection = Connection::session().await?;
        
        let proxy = StatusNotifierItemProxy::builder(&connection)
            .destination(service)?
            .path(object_path)?
            .build()
            .await?;

        proxy.activate(x, y).await?;
        Ok::<(), anyhow::Error>(())
    })
    .await;
    
    match result {
        Ok(r) => r,
        Err(_) => Err(anyhow::anyhow!("Timeout activating tray item {}", service)),
    }
}

pub async fn secondary_activate_item(service: &str, object_path: &str, x: i32, y: i32) -> anyhow::Result<()> {
    let timeout = std::time::Duration::from_secs(2);
    
    let result = tokio::time::timeout(timeout, async {
        let connection = Connection::session().await?;
        
        let proxy = StatusNotifierItemProxy::builder(&connection)
            .destination(service)?
            .path(object_path)?
            .build()
            .await?;

        proxy.secondary_activate(x, y).await?;
        Ok::<(), anyhow::Error>(())
    })
    .await;
    
    match result {
        Ok(r) => r,
        Err(_) => Err(anyhow::anyhow!("Timeout secondary activating tray item {}", service)),
    }
}
