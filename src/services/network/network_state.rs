#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Wifi,
    Ethernet,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkState {
    pub connected: bool,
    pub connection_type: ConnectionType,
    pub signal_strength: u8,  // 0-100, only relevant for WiFi
    pub ssid: Option<String>, // WiFi SSID if connected
    pub vpn_active: bool,     // true if VPN connection is active
}

#[derive(Debug, Clone)]
pub struct VpnConnection {
    pub id: String,
    pub name: String,
    pub vpn_type: String, // "openvpn", "wireguard", etc.
    pub active: bool,
    pub path: String, // D-Bus path for the connection
}

impl NetworkState {
    pub fn get_icon_name(&self) -> &'static str {
        if !self.connected {
            match self.connection_type {
                ConnectionType::Ethernet => "network-eternet-disconnected",
                _ => "network-eternet-disconnected",
            }
        } else {
            match self.connection_type {
                ConnectionType::Ethernet => {
                    if self.vpn_active {
                        "network-eternet-secure"
                    } else {
                        "network-eternet-unsecure"
                    }
                }
                ConnectionType::Wifi => {
                    let signal_level = if self.signal_strength > 75 {
                        "high"
                    } else if self.signal_strength > 50 {
                        "good"
                    } else if self.signal_strength > 25 {
                        "medium"
                    } else {
                        "low"
                    };

                    // Format: network-wifi-{signal}-{secure/unsecure}
                    // Ex: network-wifi-high-secure
                    match (signal_level, self.vpn_active) {
                        ("high", true) => "network-wifi-high-secure",
                        ("high", false) => "network-wifi-high-unsecure",
                        ("good", true) => "network-wifi-good-secure",
                        ("good", false) => "network-wifi-good-unsecure",
                        ("medium", true) => "network-wifi-medium-secure",
                        ("medium", false) => "network-wifi-medium-unsecure",
                        ("low", true) => "network-wifi-low-secure",
                        ("low", false) => "network-wifi-low-unsecure",
                        _ => "network-wifi-low-unsecure",
                    }
                }
                ConnectionType::None => "network-eternet-disconnected",
            }
        }
    }
}
