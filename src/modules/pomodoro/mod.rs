use crate::services::{PomodoroService, PomodoroState};
use crate::theme::*;
use gpui::{div, prelude::*, rgb};

pub struct PomodoroModule {
    service: PomodoroService,
}

impl PomodoroModule {
    pub fn new() -> Self {
        Self {
            service: PomodoroService::new(),
        }
    }

    pub fn get_service_mut(&mut self) -> &mut PomodoroService {
        &mut self.service
    }

    pub fn auto_transition(&mut self) {
        self.service.auto_transition();
    }

    pub fn render(&self) -> impl IntoElement {
        let (pomodoro_icon, pomodoro_color) = match self.service.get_state() {
            PomodoroState::Idle => ("🍅", POLAR3),
            PomodoroState::Work | PomodoroState::WorkPaused => ("🍅", RED),
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => ("☕", YELLOW),
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => ("🌴", GREEN),
        };

        div()
            .w_12()
            .h_12()
            .bg(rgb(pomodoro_color))
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .text_color(rgb(POLAR0))
            .text_xs()
            .child(pomodoro_icon)
            .child(self.service.format_time())
    }
}
