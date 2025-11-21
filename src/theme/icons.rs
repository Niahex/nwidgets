// src/theme/icons.rs

use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Icons {
    // Bluetooth
    pub bluetooth_off: &'static str,
    pub bluetooth_connected: &'static str,
    pub bluetooth_on: &'static str,
    // Volume
    pub volume_mute: &'static str,
    pub volume_low: &'static str,
    pub volume_high: &'static str,
    // Pomodoro/Timer
    pub timer_outline: &'static str,
    pub timer: &'static str,
    pub coffee: &'static str,
    pub beach: &'static str,
    // Workspace/Window
    pub window: &'static str,
    pub desktop: &'static str,
    pub monitor: &'static str,
    // Notification
    pub bell: &'static str,
    pub bell_slash: &'static str,
    // System tray
    pub systray: &'static str,
    // Time/Date
    pub clock: &'static str,
    pub calendar: &'static str,
    // Lock
    pub capslock: &'static str,
    pub numlock: &'static str,
    pub lock: &'static str,
    pub unlock: &'static str,
    // Media
    pub play: &'static str,
    pub pause: &'static str,
    pub stop: &'static str,
    // Network
    pub wifi: &'static str,
    pub wifi_off: &'static str,
    pub ethernet: &'static str,
    // Power
    pub battery_full: &'static str,
    pub battery_half: &'static str,
    pub battery_low: &'static str,
    pub battery_empty: &'static str,
    pub power: &'static str,
    // Status
    pub check: &'static str,
    pub error: &'static str,
    pub warning: &'static str,
    pub info: &'static str,
    // Arrow/Navigation
    pub arrow_up: &'static str,
    pub arrow_down: &'static str,
    pub arrow_left: &'static str,
    pub arrow_right: &'static str,
    // App specific
    pub vesktop: &'static str,
    pub firefox: &'static str,
    pub zed: &'static str,
    pub davinci: &'static str,
    pub vlc: &'static str,
    pub password: &'static str,
    pub launcher: &'static str,
    pub nixos: &'static str,
    pub steam: &'static str,
    pub game: &'static str,
    pub terminal: &'static str,
    pub inkscape: &'static str,
    pub stream: &'static str,
    // Dictation
    pub microphone: &'static str,
    pub microphone_slash: &'static str,
    // AI Chat
    pub person: &'static str,
    pub robot: &'static str,
    pub refresh: &'static str,
    pub clipboard: &'static str,
    pub edit: &'static str,
    pub code: &'static str,
    pub close: &'static str,
}

impl Icons {
    fn new() -> Self {
        Self {
            bluetooth_off: "󰂯",
            bluetooth_connected: "󰂱",
            bluetooth_on: "󰂲",
            volume_mute: "",
            volume_low: "",
            volume_high: "",
            timer_outline: "󰔛",
            timer: "󱎫",
            coffee: "󰅶",
            beach: "󰂒",
            window: "",
            desktop: "",
            monitor: "󰍹",
            bell: "",
            bell_slash: "",
            systray: "",
            clock: "",
            calendar: "",
            capslock: "󰪛",
            numlock: "",
            lock: "",
            unlock: "",
            play: "",
            pause: "",
            stop: "",
            wifi: "",
            wifi_off: "󰤭",
            ethernet: "",
            battery_full: "",
            battery_half: "",
            battery_low: "",
            battery_empty: "",
            power: "",
            check: "",
            error: "",
            warning: "",
            info: "",
            arrow_up: "",
            arrow_down: "",
            arrow_left: "",
            arrow_right: "",
            vesktop: "",
            firefox: "󰈹",
            zed: "",
            davinci: "",
            vlc: "󰕼",
            password: "󰟵",
            launcher: "󱓞",
            nixos: "",
            steam: "",
            game: "󰊗",
            terminal: "",
            inkscape: "",
            stream: "󰕵",
            microphone: "󰍬",
            microphone_slash: "󰍭",
            person: "",
            robot: "󰚩",
            refresh: "",
            clipboard: "",
            edit: "",
            code: "",
            close: "",
        }
    }

    pub fn get_for_class(&self, class_name: &str) -> &'static str {
        let class = class_name.to_lowercase();
        match class.as_str() {
            // App specific
            c if c.contains("vesktop") || c.contains("discord") => self.vesktop,
            c if c.contains("zen") => self.firefox,
            c if c.contains("zed") => self.zed,
            c if c.contains("davinci") => self.davinci,
            c if c.contains("vlc") => self.vlc,
            c if c.contains("1password") || c.contains("keepass") || c.contains("bitwarden") => {
                self.password
            }
            c if c.contains("rofi") || c.contains("nlauncher") => self.launcher,
            c if c.contains("steam") => self.steam,
            c if c.contains("game") || c.contains("minecraft") || c.contains("lutris") => self.game,
            c if c.contains("kitty")
                || c.contains("alacritty")
                || c.contains("wezterm")
                || c.contains("terminal") =>
            {
                self.terminal
            }
            c if c.contains("inkscape") => self.inkscape,
            c if c.contains("obs") || c.contains("stream") => self.stream,
            _ => self.window,
        }
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

pub static ICONS: Lazy<Icons> = Lazy::new(Icons::new);
