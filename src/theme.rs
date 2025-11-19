// Nord Dark Color Palette
// https://www.nordtheme.com/
#[allow(dead_code)]
pub mod colors {
    // Polar Night (Dark backgrounds)
    pub const POLAR0: u32 = 0x2e3440;
    pub const POLAR1: u32 = 0x3b4252;
    pub const POLAR2: u32 = 0x434c5e;
    pub const POLAR3: u32 = 0x4c566a;

    // Snow Storm (Light text)
    pub const SNOW0: u32 = 0xd8dee9;
    pub const SNOW1: u32 = 0xe5e9f0;
    pub const SNOW2: u32 = 0xeceff4;

    // Frost (Blues)
    pub const FROST0: u32 = 0x8fbcbb;
    pub const FROST1: u32 = 0x88c0d0;
    pub const FROST2: u32 = 0x81a1c1;
    pub const FROST3: u32 = 0x5e81ac;

    // Aurora (Accent colors)
    pub const RED: u32 = 0xbf616a;
    pub const ORANGE: u32 = 0xd08770;
    pub const YELLOW: u32 = 0xebcb8b;
    pub const GREEN: u32 = 0xa3be8c;
    pub const PURPLE: u32 = 0xb48ead;
}

// Re-export colors at root level for backwards compatibility
pub use colors::*;

// Nerd Font Icons Dictionary
#[allow(dead_code)]
pub mod icons {
    // Bluetooth icons
    pub const BLUETOOTH_OFF: &str = "󰂯"; // nf-md-bluetooth_off
    pub const BLUETOOTH_CONNECTED: &str = "󰂱"; // nf-md-bluetooth_connected
    pub const BLUETOOTH_ON: &str = "󰂲"; // nf-md-bluetooth

    // Volume icons
    pub const VOLUME_MUTE: &str = ""; // nf-fa-volume_off
    pub const VOLUME_LOW: &str = ""; // nf-fa-volume_down
    pub const VOLUME_HIGH: &str = ""; // nf-fa-volume_up

    // Pomodoro/Timer icons
    pub const TIMER_OUTLINE: &str = "󰔛"; // nf-md-timer_outline
    pub const TIMER: &str = "󱎫"; // nf-md-timer
    pub const COFFEE: &str = "󰅶"; // nf-md-coffee
    pub const BEACH: &str = "󰂒"; // nf-md-beach

    // Workspace/Window icons
    pub const WINDOW: &str = ""; // nf-fa-window_maximize
    pub const DESKTOP: &str = ""; // nf-fa-desktop
    pub const MONITOR: &str = "󰍹"; // nf-md-monitor

    // Notification icons
    pub const BELL: &str = ""; // nf-fa-bell
    pub const BELL_SLASH: &str = ""; // nf-fa-bell_slash

    // System tray icons
    pub const SYSTRAY: &str = ""; // nf-fa-tasks

    // Time/Date icons
    pub const CLOCK: &str = ""; // nf-fa-clock_o
    pub const CALENDAR: &str = ""; // nf-fa-calendar

    // Lock icons
    pub const CAPSLOCK: &str = "󰪛"; // nf-fa-lock
    pub const NUMLOCK: &str = ""; // nf-fa-hashtag
    pub const LOCK: &str = ""; // nf-fa-lock
    pub const UNLOCK: &str = ""; // nf-fa-unlock

    // Media icons
    pub const PLAY: &str = ""; // nf-fa-play
    pub const PAUSE: &str = ""; // nf-fa-pause
    pub const STOP: &str = ""; // nf-fa-stop

    // Network icons
    pub const WIFI: &str = ""; // nf-fa-wifi
    pub const WIFI_OFF: &str = "󰤭"; // nf-fa-wifi (with slash)
    pub const ETHERNET: &str = ""; // nf-fa-server

    // Power icons
    pub const BATTERY_FULL: &str = ""; // nf-fa-battery_full
    pub const BATTERY_HALF: &str = ""; // nf-fa-battery_half
    pub const BATTERY_LOW: &str = ""; // nf-fa-battery_quarter
    pub const BATTERY_EMPTY: &str = ""; // nf-fa-battery_empty
    pub const POWER: &str = ""; // nf-fa-power_off

    // Status icons
    pub const CHECK: &str = ""; // nf-fa-check
    pub const ERROR: &str = ""; // nf-fa-times
    pub const WARNING: &str = ""; // nf-fa-exclamation_triangle
    pub const INFO: &str = ""; // nf-fa-info_circle

    // Arrow/Navigation icons
    pub const ARROW_UP: &str = ""; // nf-fa-arrow_up
    pub const ARROW_DOWN: &str = ""; // nf-fa-arrow_down
    pub const ARROW_LEFT: &str = ""; // nf-fa-arrow_left
    pub const ARROW_RIGHT: &str = ""; // nf-fa-arrow_right

    pub const VESKTOP: &str = "";
    pub const FIREFOX: &str = "󰈹";
    pub const ZED: &str = "";
    pub const DAVINCI: &str = "";
    pub const VLC: &str = "󰕼";
    pub const PASSWORD: &str = "󰟵";
    pub const LAUNCHER: &str = "󱓞";
    pub const NIXOS: &str = "";
    pub const STEAM: &str = "";
    pub const GAME: &str = "󰊗";
    pub const TERMINAL: &str = "";
    pub const INKSCAPE: &str = "";
    pub const STREAM: &str = "󰕵";
}
