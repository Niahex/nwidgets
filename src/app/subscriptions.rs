use gpui::*;

use crate::services::system::{HyprlandService};
use crate::widgets::chat::ChatService;
use crate::widgets::launcher::LauncherService;
use crate::services::ui::{NotificationAdded, NotificationService};
use crate::widgets::notifications::{NotificationsStateChanged, NotificationsWindowManager};

use crate::widgets::chat::window;
use crate::widgets::launcher::window as launcher;

pub fn setup_all(
    cx: &mut App,
    chat_service: Entity<ChatService>,
    launcher_service: Entity<LauncherService>,
    notif_service: Entity<NotificationService>,
) {
    setup_chat(cx, chat_service);
    setup_launcher(cx, launcher_service);
    setup_notifications(cx, notif_service);
}

fn setup_chat(cx: &mut App, chat_service: Entity<ChatService>) {
    cx.subscribe(&chat_service, window::on_toggle).detach();
    cx.subscribe(&HyprlandService::global(cx), window::on_fullscreen).detach();
    cx.subscribe(&HyprlandService::global(cx), window::on_workspace_change).detach();
    cx.subscribe(&chat_service, window::on_navigate).detach();
}

fn setup_launcher(cx: &mut App, launcher_service: Entity<LauncherService>) {
    cx.subscribe(&launcher_service, launcher::on_toggle).detach();
}

fn setup_notifications(cx: &mut App, notif_service: Entity<NotificationService>) {
    let manager = std::sync::Arc::new(parking_lot::Mutex::new(NotificationsWindowManager::new()));
    let manager_clone = manager.clone();

    cx.subscribe(&notif_service, move |_service, _event: &NotificationAdded, cx| {
        let mut mgr = manager_clone.lock();
        if let Some(widget) = mgr.open_window(cx) {
            let mgr2 = manager_clone.clone();
            cx.subscribe(&widget, move |_widget, event: &NotificationsStateChanged, cx| {
                if !event.has_notifications {
                    mgr2.lock().close_window(cx);
                }
            }).detach();
        }
    }).detach();
}
