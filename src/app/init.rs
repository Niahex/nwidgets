use gpui::*;

use crate::services::hardware::{BluetoothService, SystemMonitorService};
use crate::services::media::{AudioService, MprisService, PomodoroService};
use crate::services::system::{ClipboardMonitor, DbusService, HyprlandService};
use crate::services::ui::{ControlCenterService, SystrayService};
use crate::services::CefService;
use crate::widgets::chat::ChatService;
use crate::widgets::launcher::LauncherService;
use crate::widgets::notifications::NotificationService;
use crate::widgets::osd::OsdService;

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
