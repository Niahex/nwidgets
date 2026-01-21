use gpui::*;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::services::ui::ClipboardMonitor;
use crate::services::launcher::{LauncherService, LauncherToggled};
use crate::widgets::launcher::LauncherWidget;

static LAUNCHER_WINDOW: once_cell::sync::OnceCell<Arc<Mutex<WindowHandle<LauncherWidget>>>> = once_cell::sync::OnceCell::new();

pub fn open(cx: &mut App, launcher_service: Entity<LauncherService>, clipboard: Entity<ClipboardMonitor>) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(0.0), y: px(0.0) },
                size: Size { width: px(1.0), height: px(1.0) },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-launcher".to_string(),
                layer: Layer::Overlay,
                anchor: Anchor::empty(),
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        },
        move |_window, cx| cx.new(|cx| LauncherWidget::new(cx, launcher_service, clipboard)),
    ).expect("Failed to create launcher window");

    LAUNCHER_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn on_toggle(service: Entity<LauncherService>, _event: &LauncherToggled, cx: &mut App) {
    let Some(window) = LAUNCHER_WINDOW.get() else { return };
    let window = window.lock();
    let visible = service.read(cx).visible;
    
    let _ = window.update(cx, |launcher, window, cx| {
        if visible {
            window.resize(size(px(700.0), px(500.0)));
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::Exclusive);
            window.set_layer(gpui::layer_shell::Layer::Overlay);
            launcher.reset();
            window.focus(launcher.focus_handle(), cx);
            cx.activate(true);
        } else {
            window.resize(size(px(1.0), px(1.0)));
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::None);
            window.set_input_region(None);
            window.set_layer(gpui::layer_shell::Layer::Background);
        }
        cx.notify();
    });
}
