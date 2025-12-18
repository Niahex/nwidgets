use gpui::prelude::*;
use gpui::*;
use crate::services::pomodoro::{PomodoroPhase, PomodoroService, PomodoroStateChanged, PomodoroStatus};
use crate::utils::Icon;

pub struct PomodoroModule {
    pomodoro: Entity<PomodoroService>,
}

impl PomodoroModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let pomodoro = PomodoroService::global(cx);

        cx.subscribe(&pomodoro, |_this, _pomodoro, _event: &PomodoroStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { pomodoro }
    }

    fn format_time(seconds: u32) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

impl Render for PomodoroModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.pomodoro.read(cx).status();
        let pomodoro = self.pomodoro.clone();

        match status {
            PomodoroStatus::Idle => {
                div()
                    .id("pomodoro-idle")
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        pomodoro.update(cx, |service, cx| {
                            service.start_work(cx);
                        });
                    })
                    .child(
                        Icon::new("play")
                            .size(px(16.))
                            .color(rgb(0xeceff4))
                    )
            }
            PomodoroStatus::Paused { .. } => {
                div()
                    .id("pomodoro-paused")
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        pomodoro.update(cx, |service, cx| {
                            service.resume(cx);
                        });
                    })
                    .child(
                        Icon::new("pause")
                            .size(px(16.))
                            .color(rgb(0xeceff4))
                    )
            }
            PomodoroStatus::Running { phase, remaining_secs } => {
                let color = match phase {
                    PomodoroPhase::Work => rgb(0xbf616a),
                    PomodoroPhase::ShortBreak => rgb(0xa3be8c),
                    PomodoroPhase::LongBreak => rgb(0x88c0d0),
                };

                div()
                    .id("pomodoro-running")
                    .text_xs()
                    .text_color(color)
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        pomodoro.update(cx, |service, cx| {
                            service.pause(cx);
                        });
                    })
                    .child(Self::format_time(remaining_secs))
            }
        }
    }
}
