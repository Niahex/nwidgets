use gpui::*;

use crate::services::system::{FullscreenChanged, HyprlandService, WorkspaceChanged};
use crate::widgets::dofustools::service::DofusToolsService;
use crate::widgets::dofustools::types::DofusToolsToggled;
use crate::widgets::dofustools::window::window_manager::get_window;

pub fn on_toggle(service: Entity<DofusToolsService>, _event: &DofusToolsToggled, cx: &mut App) {
    let Some(window) = get_window() else {
        return;
    };
    let window = window.lock();
    let visible = service.read(cx).visible;
    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();

    let _ = window.update(cx, |dofustools, window, cx| {
        if visible {
            let height = if fullscreen { 1400 } else { 1330 };
            window.resize(size(px(600.0), px(height as f32)));
            dofustools.resize_browser(600, height, cx);
            window.set_margin(
                if fullscreen { 0 } else { 40 },
                if fullscreen { 0 } else { 10 },
                if fullscreen { 0 } else { 20 },
                0,
            );
            window.set_exclusive_edge(gpui::layer_shell::Anchor::RIGHT);
            window.set_exclusive_zone(if fullscreen { 0 } else { 600 });
            window.set_layer(if fullscreen {
                gpui::layer_shell::Layer::Overlay
            } else {
                gpui::layer_shell::Layer::Top
            });
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::OnDemand);
            cx.activate(true);
        } else {
            window.set_exclusive_zone(0);
            window.resize(size(px(1.0), px(1.0)));
            window.set_layer(gpui::layer_shell::Layer::Background);
        }
        cx.notify();
    });
}

pub fn on_fullscreen(_hypr: Entity<HyprlandService>, event: &FullscreenChanged, cx: &mut App) {
    let dofustools_service = DofusToolsService::global(cx);
    if dofustools_service.read(cx).visible && event.0 {
        dofustools_service.update(cx, |dt, cx| dt.toggle(cx));
    }
}

pub fn on_workspace_change(
    _hypr: Entity<HyprlandService>,
    _event: &WorkspaceChanged,
    cx: &mut App,
) {
    let fullscreen = HyprlandService::global(cx).read(cx).has_fullscreen();
    let dofustools_service = DofusToolsService::global(cx);
    if dofustools_service.read(cx).visible && fullscreen {
        dofustools_service.update(cx, |dt, cx| dt.toggle(cx));
    }
}
