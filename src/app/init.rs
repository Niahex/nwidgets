use gpui::*;

use crate::services::{
    audio::AudioService, bluetooth::BluetoothService, cef::CefService,
    chat::ChatService, clipboard::ClipboardMonitor, control_center::ControlCenterService,
    dbus::DbusService, hyprland::HyprlandService, launcher::LauncherService,
    mpris::MprisService, notifications::NotificationService, osd::OsdService,
    pomodoro::PomodoroService, system_monitor::SystemMonitorService, systray::SystrayService,
};

pub fn initialize_all(cx: &mut App) -> (Entity<ClipboardMonitor>, Entity<OsdService>) {
    HyprlandService::init(cx);
    AudioService::init(cx);
    BluetoothService::init(cx);
    crate::services::network::init_network_services(cx);
    SystemMonitorService::init(cx);
    MprisService::init(cx);
    PomodoroService::init(cx);
    SystrayService::init(cx);
    CefService::init(cx);
    DbusService::init(cx);
    ChatService::init(cx);
    LauncherService::init(cx);
    let clipboard = ClipboardMonitor::init(cx);
    NotificationService::init(cx);
    let osd = OsdService::init(cx);
    ControlCenterService::init(cx);
    crate::services::cef::CefService::init(cx);
    
    (clipboard, osd)
}

pub fn get_globals(cx: &mut App) -> (Entity<ChatService>, Entity<LauncherService>, Entity<NotificationService>) {
    (
        ChatService::global(cx),
        LauncherService::global(cx),
        NotificationService::global(cx),
    )
}
