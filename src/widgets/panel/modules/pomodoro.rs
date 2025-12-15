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
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .id("pomodoro-start")
                            .flex()
                            .gap_1()
                            .items_center()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(0xeceff4)) // $snow2
                            .bg(rgba(0x3b425266)) // $polar1 with opacity
                            .hover(|style| style.bg(rgba(0x4c566a99))) // $polar3
                            .cursor_pointer()
                            .on_click(move |_event, _window, cx| {
                                pomodoro.update(cx, |service, cx| {
                                    service.start_work(cx);
                                });
                            })
                            .child(
                                Icon::new("coffee")
                                    .size(px(16.))
                                    .color(rgb(0x88c0d0)) // $frost1
                            )
                            .child("Start")
                    )
            }
            PomodoroStatus::Running { ref phase, remaining_secs } |
            PomodoroStatus::Paused { ref phase, remaining_secs } => {
                let is_running = matches!(status, PomodoroStatus::Running { .. });
                let phase_text = match phase {
                    PomodoroPhase::Work => "Work",
                    PomodoroPhase::ShortBreak => "Break",
                    PomodoroPhase::LongBreak => "Long Break",
                };
                let pomodoro_pause = pomodoro.clone();
                let pomodoro_stop = pomodoro.clone();

                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .text_sm()
                    .text_color(rgb(0xeceff4)) // $snow2
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .items_center()
                            .child(
                                Icon::new("coffee")
                                    .size(px(16.))
                                    .color(rgb(0x88c0d0)) // $frost1
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("{} {}", phase_text, Self::format_time(remaining_secs)))
                            )
                    )
                    .child(
                        div()
                            .id("pomodoro-pause-resume")
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .hover(|style| style.bg(rgba(0x4c566a80))) // $polar3 with opacity
                            .cursor_pointer()
                            .on_click(move |_event, _window, cx| {
                                pomodoro_pause.update(cx, |service, cx| {
                                    if is_running {
                                        service.pause(cx);
                                    } else {
                                        service.resume(cx);
                                    }
                                });
                            })
                            .child(
                                Icon::new(if is_running { "pause" } else { "play" })
                                    .size(px(14.))
                                    .color(rgb(0xeceff4))
                            )
                    )
                    .child(
                        div()
                            .id("pomodoro-stop")
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .hover(|style| style.bg(rgba(0x4c566a80))) // $polar3 with opacity
                            .cursor_pointer()
                            .on_click(move |_event, _window, cx| {
                                pomodoro_stop.update(cx, |service, cx| {
                                    service.stop(cx);
                                });
                            })
                            .child(
                                Icon::new("recording-stopped")
                                    .size(px(14.))
                                    .color(rgb(0xeceff4))
                            )
                    )
            }
        }
    }
}
