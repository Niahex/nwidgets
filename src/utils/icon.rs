use gpui::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
    pub fn path(&self) -> Arc<str> {
        let file_stem = match self {
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
        };
        format!("assets/{file_stem}.svg").into()
    }
}

#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(16.),
            color: None,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut svg_element = svg()
            .path(self.name.path())
            .size(self.size);

        if let Some(color) = self.color {
            svg_element = svg_element.text_color(color);
        }

        svg_element
    }
}
