use gpui::*;

use crate::services::system::{FullscreenChanged, HyprlandService, WorkspaceChanged};
use crate::widgets::chat::service::{save_url, ChatService};
use crate::widgets::chat::types::{ChatNavigate, ChatToggled};
use crate::widgets::chat::window::window_manager::get_window;

pub fn on_toggle(service: Entity<ChatService>, _event: &ChatToggled, cx: &mut App) {
    let Some(window) = get_window() else {
        return;
    };
    let window = window.lock();
    let visible = service.read(cx).visible;
    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();

    let _ = window.update(cx, |chat, window, cx| {
        if visible {
            let height = if fullscreen { 1440 } else { 1370 };
            window.resize(size(px(600.0), px(height as f32)));
            chat.resize_browser(600, height, cx);
            chat.focus_input(cx);
            window.set_margin(
                if fullscreen { 0 } else { 40 },
                0,
                if fullscreen { 0 } else { 20 },
                if fullscreen { 0 } else { 10 },
            );
            window.set_exclusive_edge(gpui::layer_shell::Anchor::LEFT);
            window.set_exclusive_zone(if fullscreen { 0 } else { 600 });
            window.set_layer(if fullscreen {
                gpui::layer_shell::Layer::Overlay
            } else {
                gpui::layer_shell::Layer::Top
            });
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::OnDemand);
            cx.activate(true);
        } else {
            if let Some(url) = chat.current_url(cx) {
                save_url(&url);
            }
            window.set_exclusive_zone(0);
            window.resize(size(px(1.0), px(1.0)));
            window.set_layer(gpui::layer_shell::Layer::Background);
        }
        cx.notify();
    });
}

pub fn on_fullscreen(_hypr: Entity<HyprlandService>, event: &FullscreenChanged, cx: &mut App) {
    let chat_service = ChatService::global(cx);
    if chat_service.read(cx).visible && event.0 {
        chat_service.update(cx, |cs, cx| cs.toggle(cx));
    }
}

pub fn on_workspace_change(
    _hypr: Entity<HyprlandService>,
    _event: &WorkspaceChanged,
    cx: &mut App,
) {
    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
    let chat_service = ChatService::global(cx);
    if chat_service.read(cx).visible && fullscreen {
        chat_service.update(cx, |cs, cx| cs.toggle(cx));
    }
}

pub fn on_navigate(_service: Entity<ChatService>, event: &ChatNavigate, cx: &mut App) {
    let Some(window) = get_window() else {
        return;
    };
    let window = window.lock();
    let _ = window.update(cx, |chat, _window, cx| {
        chat.navigate(&event.url, cx);
    });
}
