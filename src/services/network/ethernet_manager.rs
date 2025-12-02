use super::network_state::ConnectionType;
use super::{ActiveConnectionProxy, DeviceProxy};
use zbus::Connection;

pub struct EthernetManager;

impl EthernetManager {
    pub async fn get_connection_type(
        connection_path: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<ConnectionType, Box<dyn std::error::Error>> {
        let connection = Connection::system().await?;
        let active_conn = ActiveConnectionProxy::builder(&connection)
            .path(connection_path)?
            .build()
            .await?;

        let conn_type = active_conn.connection_type().await?;
        let devices = active_conn.devices().await?;

        if conn_type == "802-11-wireless" {
            return Ok(ConnectionType::Wifi);
        }

        // Check device types for ethernet
        for device_path in devices {
            let device = DeviceProxy::builder(&connection)
                .path(&device_path)?
                .build()
                .await?;

            let device_type = device.device_type().await?;

            // Device type 1 = Ethernet, 2 = WiFi
            match device_type {
                1 => return Ok(ConnectionType::Ethernet),
                2 => return Ok(ConnectionType::Wifi),
                _ => continue,
            }
        }

        Ok(ConnectionType::None)
    }
}
