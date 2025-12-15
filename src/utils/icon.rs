use gpui::*;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Cache global des icônes SVG chargées
static ICON_CACHE: Lazy<RwLock<HashMap<String, Arc<str>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Répertoire des assets (peut être overridé via variable d'environnement)
fn assets_dir() -> PathBuf {
    std::env::var("NWIDGETS_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"))
}

/// Composant Icon qui charge dynamiquement les SVG depuis le dossier assets/
///
/// Utilisation:
/// ```rust
/// Icon::new("spotify")          // Charge assets/spotify.svg
/// Icon::new("sink-high")        // Charge assets/sink-high.svg
///     .size(px(24.))
///     .color(rgb(0xeceff4))
/// ```
#[derive(IntoElement)]
pub struct Icon {
    name: String,
    size: Pixels,
    color: Option<Hsla>,
}

impl Icon {
    /// Crée une nouvelle icône depuis un nom de fichier (sans l'extension .svg)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: px(16.),
            color: None,
        }
    }

    /// Définit la taille de l'icône
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Définit la couleur de l'icône
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Récupère le chemin de l'icône (avec cache)
    fn get_path(&self) -> Arc<str> {
        // Check cache first
        {
            let cache = ICON_CACHE.read();
            if let Some(path) = cache.get(&self.name) {
                return path.clone();
            }
        }

        // Not in cache, build path and cache it
        let path = format!("{}/{}.svg", assets_dir().display(), self.name);
        let path_arc: Arc<str> = path.into();

        // Store in cache
        {
            let mut cache = ICON_CACHE.write();
            cache.insert(self.name.clone(), path_arc.clone());
        }

        path_arc
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let path = self.get_path();
        let mut svg_element = svg().path(path).size(self.size);

        if let Some(color) = self.color {
            svg_element = svg_element.text_color(color);
        }

        svg_element
    }
}

// ========================================
// API Legacy pour rétro-compatibilité
// ========================================

/// Enum legacy pour les icônes existantes (rétro-compatibilité)
/// Préférez utiliser Icon::new("nom-fichier") directement
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[deprecated(
    note = "Utilisez Icon::new(\"nom-fichier\") directement au lieu de IconName::Variant"
)]
pub enum IconName {
    // Audio
    SinkHigh,
    SinkMedium,
    SinkLow,
    SinkMuted,
    SinkNone,
    SinkZero,
    SourceHigh,
    SourceMedium,
    SourceLow,
    SourceMuted,
    SourcePaused,
    SourceProcessing,
    SourceRecorder,
    SourceZero,

    // Bluetooth
    Bluetooth,
    BluetoothActive,
    BluetoothDisabled,
    BluetoothPaired,

    // Network
    Network,
    NetworkVpn,
    NetworkEternetConnected,
    NetworkEternetDisconnected,
    NetworkEternetSecure,
    NetworkEternetUnsecure,
    NetworkWifiHigh,
    NetworkWifiHighSecure,
    NetworkWifiHighUnsecure,
    NetworkWifiGood,
    NetworkWifiGoodSecure,
    NetworkWifiGoodUnsecure,
    NetworkWifiMedium,
    NetworkWifiMediumSecure,
    NetworkWifiMediumUnsecure,
    NetworkWifiLow,
    NetworkWifiLowSecure,
    NetworkWifiLowUnsecure,

    // Media controls
    Play,
    Pause,

    // Notifications
    Notification,
    Error,
    Warning,
    Info,
    Question,
    Help,

    // Recording
    RecordingCountdown,
    RecordingPaused,
    RecordingRecording,
    RecordingStopped,

    // Misc
    Calendar,
    Capslock,
    CapslockOff,
    CapslockOn,
    Clipboard,
    Clip,
    Coffee,
    Copy,
    Pin,
    Unpin,
    Search,
    Send,
    Terminal,

    // Apps
    Spotify,
    SpotifyIndicator,
    Discord,
    Firefox,
    FirefoxWhite,
    Brave,
    Steam,
    SteamTray,
    Thunderbird,
    Vlc,

    // Arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUpDouble,
    ArrowDownDouble,
    ArrowLeftDouble,
    ArrowRightDouble,
}

impl IconName {
    /// Convertit l'enum en nom de fichier
    pub fn as_str(&self) -> &'static str {
        match self {
            IconName::SinkHigh => "sink-high",
            IconName::SinkMedium => "sink-medium",
            IconName::SinkLow => "sink-low",
            IconName::SinkMuted => "sink-muted",
            IconName::SinkNone => "sink-none",
            IconName::SinkZero => "sink-zero",
            IconName::SourceHigh => "source-high",
            IconName::SourceMedium => "source-medium",
            IconName::SourceLow => "source-low",
            IconName::SourceMuted => "source-muted",
            IconName::SourcePaused => "source-paused",
            IconName::SourceProcessing => "source-processing",
            IconName::SourceRecorder => "source-recorder",
            IconName::SourceZero => "source-zero",
            IconName::Bluetooth => "bluetooth",
            IconName::BluetoothActive => "bluetooth-active",
            IconName::BluetoothDisabled => "bluetooth-disabled",
            IconName::BluetoothPaired => "bluetooth-paired",
            IconName::Network => "network",
            IconName::NetworkVpn => "network-vpn",
            IconName::NetworkEternetConnected => "network-eternet-connected",
            IconName::NetworkEternetDisconnected => "network-eternet-disconnected",
            IconName::NetworkEternetSecure => "network-eternet-secure",
            IconName::NetworkEternetUnsecure => "network-eternet-unsecure",
            IconName::NetworkWifiHigh => "network-wifi-high",
            IconName::NetworkWifiHighSecure => "network-wifi-high-secure",
            IconName::NetworkWifiHighUnsecure => "network-wifi-high-unsecure",
            IconName::NetworkWifiGood => "network-wifi-good",
            IconName::NetworkWifiGoodSecure => "network-wifi-good-secure",
            IconName::NetworkWifiGoodUnsecure => "network-wifi-good-unsecure",
            IconName::NetworkWifiMedium => "network-wifi-medium",
            IconName::NetworkWifiMediumSecure => "network-wifi-medium-secure",
            IconName::NetworkWifiMediumUnsecure => "network-wifi-medium-unsecure",
            IconName::NetworkWifiLow => "network-wifi-low",
            IconName::NetworkWifiLowSecure => "network-wifi-low-secure",
            IconName::NetworkWifiLowUnsecure => "network-wifi-low-unsecure",
            IconName::Play => "play",
            IconName::Pause => "pause",
            IconName::Notification => "notification",
            IconName::Error => "error",
            IconName::Warning => "warning",
            IconName::Info => "info",
            IconName::Question => "question",
            IconName::Help => "help",
            IconName::RecordingCountdown => "recording-countdown",
            IconName::RecordingPaused => "recording-paused",
            IconName::RecordingRecording => "recording-recording",
            IconName::RecordingStopped => "recording-stopped",
            IconName::Calendar => "calendar",
            IconName::Capslock => "capslock",
            IconName::CapslockOff => "capslock-off",
            IconName::CapslockOn => "capslock-on",
            IconName::Clipboard => "clipboard",
            IconName::Clip => "clip",
            IconName::Coffee => "coffee",
            IconName::Copy => "copy",
            IconName::Pin => "pin",
            IconName::Unpin => "unpin",
            IconName::Search => "search",
            IconName::Send => "send",
            IconName::Terminal => "terminal",
            IconName::Spotify => "spotify",
            IconName::SpotifyIndicator => "spotify-indicator",
            IconName::Discord => "discord",
            IconName::Firefox => "firefox",
            IconName::FirefoxWhite => "firefox-white",
            IconName::Brave => "brave",
            IconName::Steam => "steam",
            IconName::SteamTray => "steam_tray",
            IconName::Thunderbird => "thunderbird",
            IconName::Vlc => "vlc",
            IconName::ArrowUp => "arrow-up",
            IconName::ArrowDown => "arrow-down",
            IconName::ArrowLeft => "arrow-left",
            IconName::ArrowRight => "arrow-right",
            IconName::ArrowUpDouble => "arrow-up-double",
            IconName::ArrowDownDouble => "arrow-down-double",
            IconName::ArrowLeftDouble => "arrow-left-double",
            IconName::ArrowRightDouble => "arrow-right-double",
        }
    }

    /// Méthode legacy pour compatibilité
    #[deprecated(note = "Utilisez Icon::new(icon_name.as_str()) à la place")]
    pub fn path(&self) -> Arc<str> {
        format!("{}/{}.svg", assets_dir().display(), self.as_str()).into()
    }
}

// Permet de convertir IconName en Icon facilement
impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Icon::new(name.as_str())
    }
}
