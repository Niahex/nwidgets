use makepad_widgets::*;

use crate::POMODORO_SERVICE;
use crate::services::media::pomodoro::PomodoroPhase;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub PomodoroModule = {{PomodoroModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.5, y: 0.5}
        spacing: 6
        padding: {left: 8, right: 8}
        cursor: Hand

        content = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }, color: (POMODORO_COLOR_DEFAULT) }
            text: "󰐊"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PomodoroModule {
    #[deref]
    view: View,

    #[rust]
    timer: Timer,
}

impl Widget for PomodoroModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(1.0);
        }

        if let Event::Startup = event {
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(1.0);
        }

        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(fe) => {
                let is_middle = fe.device.mouse_button()
                    .map(|b| b.is_middle())
                    .unwrap_or(false);
                
                if is_middle {
                    POMODORO_SERVICE.reset();
                } else if fe.tap_count == 1 {
                    POMODORO_SERVICE.toggle();
                }
                self.sync_from_service(cx);
            }
            _ => {}
        }
    }
}

impl PomodoroModule {
    fn sync_from_service(&mut self, cx: &mut Cx) {
        let state = POMODORO_SERVICE.get_state();
        
        if state.is_running {
            let minutes = state.remaining_seconds / 60;
            let secs = state.remaining_seconds % 60;
            let time_str = format!("{:02}:{:02}", minutes, secs);
            
            let color = match state.phase {
                PomodoroPhase::Work => live!{
                    draw_text: { color: (POMODORO_COLOR_WORK) }
                },
                PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => live!{
                    draw_text: { color: (POMODORO_COLOR_BREAK) }
                },
            };
            
            self.view.label(ids!(content)).set_text(cx, &time_str);
            self.view.label(ids!(content)).apply_over(cx, color);
        } else {
            let icon_text = if state.has_started {
                "󰏤"
            } else {
                match state.phase {
                    PomodoroPhase::Work => "󰐊",
                    PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => "󰝛",
                }
            };
            
            self.view.label(ids!(content)).set_text(cx, icon_text);
            self.view.label(ids!(content)).apply_over(cx, live!{
                draw_text: { color: (POMODORO_COLOR_DEFAULT) }
            });
        }

        self.view.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum PomodoroAction {
    None,
    Toggle,
    Reset,
}
