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
            if fullscreen {
                window.resize(size(px(600.0), px(1440.0)));
                chat.resize_browser(600, 1440, cx);
                window.set_margin(0, 0, 0, 0);
                window.set_exclusive_zone(0);
                window.set_layer(gpui::layer_shell::Layer::Overlay);
            } else {
                window.resize(size(px(600.0), px(1370.0)));
                chat.resize_browser(600, 1370, cx);
                window.set_margin(40, 0, 20, 10);
                window.set_exclusive_zone(600);
                window.set_layer(gpui::layer_shell::Layer::Top);
            }
            window.set_exclusive_edge(gpui::layer_shell::Anchor::LEFT);
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::OnDemand);
            chat.focus_input(cx);
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
    
    if !chat_service.read(cx).visible {
        return;
    }
    
    let Some(window) = get_window() else {
        return;
    };
    let window = window.lock();
    
    let _ = window.update(cx, |chat, window, cx| {
        if event.0 {
            window.resize(size(px(600.0), px(1440.0)));
            chat.resize_browser(600, 1440, cx);
            window.set_margin(0, 0, 0, 0);
            window.set_exclusive_zone(0);
            window.set_layer(gpui::layer_shell::Layer::Overlay);
        } else {
            window.resize(size(px(600.0), px(1370.0)));
            chat.resize_browser(600, 1370, cx);
            window.set_margin(40, 0, 20, 10);
            window.set_exclusive_zone(600);
            window.set_layer(gpui::layer_shell::Layer::Top);
        }
        cx.notify();
    });
}

pub fn on_workspace_change(
    _hypr: Entity<HyprlandService>,
    _event: &WorkspaceChanged,
    cx: &mut App,
) {
    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
    let chat_service = ChatService::global(cx);
    
    if !chat_service.read(cx).visible {
        return;
    }
    
    let Some(window) = get_window() else {
        return;
    };
    let window = window.lock();
    
    let _ = window.update(cx, |chat, window, cx| {
        if fullscreen {
            window.resize(size(px(600.0), px(1440.0)));
            chat.resize_browser(600, 1440, cx);
            window.set_margin(0, 0, 0, 0);
            window.set_exclusive_zone(0);
            window.set_layer(gpui::layer_shell::Layer::Overlay);
        } else {
            window.resize(size(px(600.0), px(1370.0)));
            chat.resize_browser(600, 1370, cx);
            window.set_margin(40, 0, 20, 10);
            window.set_exclusive_zone(600);
            window.set_layer(gpui::layer_shell::Layer::Top);
        }
        cx.notify();
    });
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
