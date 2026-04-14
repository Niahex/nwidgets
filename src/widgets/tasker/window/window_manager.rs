use gpui::*;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

use crate::widgets::panel::modules::pomodoro::PomodoroService;
use crate::widgets::tasker::widget::TaskWindow;
use crate::widgets::tasker::{TaskSelected, TaskService, TaskWindowToggled};

static TASK_WINDOW: OnceLock<Arc<Mutex<WindowHandle<TaskWindow>>>> = OnceLock::new();

pub fn open(cx: &mut App) {
    use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

    let window = match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point {
                    x: px(0.0),
                    y: px(0.0),
                },
                size: Size {
                    width: px(1.0),
                    height: px(1.0),
                },
            })),
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: Some(WindowDecorations::Client),
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "nwidgets-pomodoro-tasks".to_string(),
                layer: Layer::Background,
                anchor: Anchor::empty(),
                exclusive_zone: None,
                margin: None,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            app_id: Some("nwidgets-pomodoro-tasks".to_string()),
            is_movable: false,
            ..Default::default()
        },
        |_window, cx| cx.new(TaskWindow::new),
    ) {
        Ok(window) => window,
        Err(e) => {
            log::error!("Failed to create task window: {}", e);
            return;
        }
    };

    TASK_WINDOW.set(Arc::new(Mutex::new(window))).ok();
}

pub fn on_toggle(service: Entity<TaskService>, _event: &TaskWindowToggled, cx: &mut App) {
    let Some(window_arc) = TASK_WINDOW.get() else {
        return;
    };
    let visible = service.read(cx).window_visible();
    let window = window_arc.lock();

    if let Err(err) = window.update(cx, |task_window, window, cx| {
        if visible {
            window.set_layer(gpui::layer_shell::Layer::Overlay);
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::Exclusive);
            window.resize(size(px(600.0), px(700.0)));
            window.focus(&task_window.focus_handle, cx);
            cx.activate(true);
        } else {
            window.set_layer(gpui::layer_shell::Layer::Background);
            window.set_input_region(None);
            window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::None);
            window.resize(size(px(1.0), px(1.0)));
        }
        cx.notify();
    }) {
        log::error!("Failed to update task window: {err}");
    }
}

pub fn get_window() -> Option<Arc<Mutex<WindowHandle<TaskWindow>>>> {
    TASK_WINDOW.get().cloned()
}

pub fn on_task_selected(service: Entity<TaskService>, event: &TaskSelected, cx: &mut App) {
    let pomodoro = PomodoroService::global(cx);
    pomodoro.update(cx, |pomodoro, cx| {
        if event.task_id.is_some() {
            pomodoro.start_work(cx);
        } else {
            pomodoro.stop(cx);
        }
    });

    if event.task_id.is_some() {
        service.update(cx, |service, cx| {
            if service.window_visible() {
                service.toggle_window(cx);
            }
        });
    }
}
