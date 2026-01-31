pub use makepad_widgets;
use makepad_widgets::*;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

use crate::services::media::pomodoro::PomodoroService;
use crate::services::system::hyprland::HyprlandService;

pub mod app;
pub mod theme;
pub mod logger;

pub mod services;
pub mod ui;
pub mod widgets;

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

pub static POMODORO_SERVICE: Lazy<PomodoroService> = Lazy::new(PomodoroService::new);
pub static HYPRLAND_SERVICE: Lazy<HyprlandService> = Lazy::new(HyprlandService::new);

pub fn live_design(cx: &mut Cx) {
    makepad_widgets::live_design(cx);

    theme::live_design(cx);
    ui::live_design(cx);
    widgets::live_design(cx);
}
