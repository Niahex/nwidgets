use zbus::{Connection, proxy};

// NetworkManager Wireless Device interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait WirelessDevice {
    #[zbus(property, name = "ActiveAccessPoint")]
    fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

// NetworkManager AccessPoint interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait AccessPoint {
    #[zbus(property, name = "Strength")]
    fn strength(&self) -> zbus::Result<u8>;

    #[zbus(property, name = "Ssid")]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
}

pub struct WifiManager;

impl WifiManager {
    pub async fn get_wifi_info(device_path: &zbus::zvariant::OwnedObjectPath) -> Result<(u8, Option<String>), Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        let wireless_device = WirelessDeviceProxy::builder(&connection)
            .path(device_path)?
            .build()
            .await?;

        let ap_path = wireless_device.active_access_point().await?;
        
        // Check if there's an active access point
        if ap_path.as_str() == "/" {
            return Ok((0, None));
        }

        let access_point = AccessPointProxy::builder(&connection)
            .path(&ap_path)?
            .build()
            .await?;

        let strength = access_point.strength().await?;
        let ssid_bytes = access_point.ssid().await?;
        let ssid = if !ssid_bytes.is_empty() {
            Some(String::from_utf8_lossy(&ssid_bytes).to_string())
        } else {
            None
        };

        Ok((strength, ssid))
    }
}
