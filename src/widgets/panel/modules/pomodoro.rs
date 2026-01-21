use crate::services::pomodoro::{
    PomodoroPhase, PomodoroService, PomodoroStateChanged, PomodoroStatus,
};
use crate::theme::ActiveTheme;
use crate::assets::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct PomodoroModule {
    pomodoro: Entity<PomodoroService>,
}

impl PomodoroModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let pomodoro = PomodoroService::global(cx);

        cx.subscribe(
            &pomodoro,
            |_this, _pomodoro, _event: &PomodoroStateChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        Self { pomodoro }
    }

    fn format_time(seconds: u32) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{mins:02}:{secs:02}")
    }
}

impl Render for PomodoroModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.pomodoro.read(cx).status();
        let pomodoro_left = self.pomodoro.clone();
        let pomodoro_middle = self.pomodoro.clone();

        let theme = cx.theme();

        let element = match status {
            PomodoroStatus::Idle => div()
                .id("pomodoro-idle")
                .cursor_pointer()
                .on_click(move |_event, _window, cx| {
                    pomodoro_left.update(cx, |service, cx| {
                        service.start_work(cx);
                    });
                })
                .child(Icon::new("play").size(px(16.)).color(theme.text)),
            PomodoroStatus::Paused { .. } => div()
                .id("pomodoro-paused")
                .cursor_pointer()
                .on_click(move |_event, _window, cx| {
                    pomodoro_left.update(cx, |service, cx| {
                        service.resume(cx);
                    });
                })
                .child(Icon::new("pause").size(px(16.)).color(theme.text)),
            PomodoroStatus::Running {
                phase,
                remaining_secs,
            } => {
                let color = match phase {
                    PomodoroPhase::Work => theme.red,
                    PomodoroPhase::ShortBreak => theme.green,
                    PomodoroPhase::LongBreak => theme.accent,
                };

                div()
                    .id("pomodoro-running")
                    .text_xs()
                    .text_color(color)
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        pomodoro_left.update(cx, |service, cx| {
                            service.pause(cx);
                        });
                    })
                    .child(Self::format_time(remaining_secs))
            }
        };

        element.on_mouse_down(MouseButton::Middle, move |_event, _window, cx| {
            pomodoro_middle.update(cx, |service, cx| {
                service.stop(cx);
            });
        })
    }
}
