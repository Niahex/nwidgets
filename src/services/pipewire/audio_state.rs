#[derive(Debug, Clone, PartialEq)]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub mic_volume: u8,
    pub mic_muted: bool,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct AudioStream {
    pub id: u32,
    pub name: String,
    pub app_name: String,
    pub volume: u8,
    pub muted: bool,
    pub window_title: Option<String>,
    pub app_icon: Option<String>,
}

impl AudioState {
    /// Retourne le nom de l'icône pour le sink (sortie audio) en fonction du volume et de l'état muted
    pub fn get_sink_icon_name(&self) -> &'static str {
        if self.muted {
            "sink-muted"
        } else if self.volume == 0 {
            "sink-zero"
        } else if self.volume < 33 {
            "sink-low"
        } else if self.volume < 66 {
            "sink-medium"
        } else {
            "sink-high"
        }
    }

    /// Retourne le nom de l'icône pour la source (entrée audio/micro) en fonction du volume et de l'état muted
    pub fn get_source_icon_name(&self) -> &'static str {
        if self.mic_muted {
            "source-muted"
        } else if self.mic_volume == 0 {
            "source-zero"
        } else if self.mic_volume < 33 {
            "source-low"
        } else if self.mic_volume < 66 {
            "source-medium"
        } else {
            "source-high"
        }
    }
}
