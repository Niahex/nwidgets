use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Params {
    #[serde(rename = "Props")]
    pub props: Option<Vec<Value>>,
    #[serde(rename = "Route")]
    pub route: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetadataEntry {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PipeWireObject {
    pub id: u32,
    #[serde(rename = "type")]
    pub type_: String,
    pub info: Option<Info>,
    pub metadata: Option<Vec<MetadataEntry>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Info {
    pub props: Option<Value>,
    pub params: Option<Params>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PropsParam {
    pub volume: Option<f32>,
    pub mute: Option<bool>,
    #[serde(rename = "channelVolumes")]
    pub channel_volumes: Option<Vec<f32>>,
}

impl PipeWireObject {
    pub fn get_prop(&self, key: &str) -> Option<String> {
        self.info
            .as_ref()?
            .props
            .as_ref()?
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_media_class(&self) -> Option<String> {
        self.get_prop("media.class")
    }

    pub fn get_node_name(&self) -> Option<String> {
        self.get_prop("node.name")
    }

    pub fn get_node_desc(&self) -> Option<String> {
        self.get_prop("node.description")
            .or_else(|| self.get_prop("node.nick"))
            .or_else(|| self.get_prop("device.description"))
            .or_else(|| self.get_prop("api.alsa.card.name"))
    }

    pub fn get_app_name(&self) -> Option<String> {
        self.get_prop("application.name")
            .or_else(|| self.get_prop("node.name"))
    }

    pub fn get_app_icon_name(&self) -> Option<String> {
        self.get_prop("application.icon_name")
            .or_else(|| self.get_prop("application.icon-name"))
            .or_else(|| self.get_prop("application.process.binary"))
    }

    /// Récupère le volume et le mute depuis les params Props ou Route
    /// Retourne (volume_percent, is_muted)
    pub fn get_volume_info(&self) -> (u8, bool) {
        if let Some(info) = &self.info {
            if let Some(params) = &info.params {
                // Essayer d'abord "Props" (utilisé par la plupart des nodes)
                if let Some(props_list) = &params.props {
                    if let Some(first_prop) = props_list.first() {
                        if let Ok(p) = serde_json::from_value::<PropsParam>(first_prop.clone()) {
                            let mute = p.mute.unwrap_or(false);
                            let raw_vol = if let Some(cv) = &p.channel_volumes {
                                cv.first().copied().unwrap_or(1.0)
                            } else {
                                p.volume.unwrap_or(1.0)
                            };
                            
                            let linear_vol = raw_vol.cbrt();
                            return ((linear_vol * 100.0) as u8, mute);
                        }
                    }
                }
                
                // Essayer "Route" (parfois utilisé pour les devices alsa/bluez)
                if let Some(route_list) = &params.route {
                     if let Some(first_route) = route_list.first() {
                          if let Ok(p) = serde_json::from_value::<PropsParam>(first_route.clone()) {
                            let mute = p.mute.unwrap_or(false);
                            let raw_vol = if let Some(cv) = &p.channel_volumes {
                                cv.first().copied().unwrap_or(1.0)
                            } else {
                                p.volume.unwrap_or(1.0)
                            };
                            let linear_vol = raw_vol.cbrt();
                            return ((linear_vol * 100.0) as u8, mute);
                        }
                     }
                }
            }
        }
        (100, false) // Par défaut
    }
}