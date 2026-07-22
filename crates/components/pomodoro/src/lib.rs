use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{Icon, Sizable};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroPhase {
    Work,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroStatus {
    Idle,
    Running { phase: PomodoroPhase, remaining_secs: u32 },
    Paused { phase: PomodoroPhase, remaining_secs: u32 },
}

pub struct PomodoroComponent {
    status: PomodoroStatus,
    work_duration_secs: u32,
    break_duration_secs: u32,
}

impl PomodoroComponent {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            status: PomodoroStatus::Idle,
            work_duration_secs: 25 * 60,
            break_duration_secs: 5 * 60,
        }
    }

    pub fn start_work(&mut self, cx: &mut Context<Self>) {
        self.status = PomodoroStatus::Running {
            phase: PomodoroPhase::Work,
            remaining_secs: self.work_duration_secs,
        };
        self.start_timer(cx);
        cx.notify();
    }

    pub fn pause(&mut self, cx: &mut Context<Self>) {
        if let PomodoroStatus::Running { phase, remaining_secs } = self.status {
            self.status = PomodoroStatus::Paused { phase, remaining_secs };
            cx.notify();
        }
    }

    pub fn resume(&mut self, cx: &mut Context<Self>) {
        if let PomodoroStatus::Paused { phase, remaining_secs } = self.status {
            self.status = PomodoroStatus::Running { phase, remaining_secs };
            self.start_timer(cx);
            cx.notify();
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.status = PomodoroStatus::Idle;
        cx.notify();
    }

    fn start_timer(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;

            let should_continue = this
                .update(cx, |this, cx| match this.status {
                    PomodoroStatus::Running { phase, remaining_secs } => {
                        if remaining_secs > 1 {
                            this.status = PomodoroStatus::Running {
                                phase,
                                remaining_secs: remaining_secs - 1,
                            };
                            cx.notify();
                            true
                        } else {
                            let (next_phase, next_secs) = match phase {
                                PomodoroPhase::Work => (PomodoroPhase::Break, this.break_duration_secs),
                                PomodoroPhase::Break => (PomodoroPhase::Work, this.work_duration_secs),
                            };
                            this.status = PomodoroStatus::Running {
                                phase: next_phase,
                                remaining_secs: next_secs,
                            };
                            cx.notify();
                            true
                        }
                    }
                    _ => false,
                })
                .unwrap_or(false);

            if !should_continue {
                break;
            }
        })
        .detach();
    }

    fn format_time(seconds: u32) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{mins:02}:{secs:02}")
    }
}

impl Render for PomodoroComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let btn = match self.status {
            PomodoroStatus::Idle => Button::new("pomodoro-btn")
                .ghost()
                .with_size(gpui_component::Size::Medium)
                .icon(Icon::new("play_arrow").size(px(30.0)))
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.start_work(cx);
                })),

            PomodoroStatus::Running { phase: _, remaining_secs } => Button::new("pomodoro-btn")
                .ghost()
                .with_size(gpui_component::Size::Medium)
                .icon(Icon::new("pause").size(px(30.0)))
                .label(Self::format_time(remaining_secs))
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.pause(cx);
                })),

            PomodoroStatus::Paused { phase: _, remaining_secs } => Button::new("pomodoro-btn")
                .ghost()
                .with_size(gpui_component::Size::Medium)
                .icon(Icon::new("play_arrow").size(px(30.0)))
                .label(Self::format_time(remaining_secs))
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.resume(cx);
                })),
        };

        div()
            .id("pomodoro-container")
            .on_mouse_down(MouseButton::Middle, cx.listener(|this, _, _window, cx| {
                this.reset(cx);
            }))
            .child(btn)
    }
}
