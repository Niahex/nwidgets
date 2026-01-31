use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub PomodoroModule = {{PomodoroModule}} {
        width: Fit, height: Fill
        flow: Row
        align: {x: 0.5, y: 0.5}
        spacing: 6
        padding: {left: 8, right: 8}
        cursor: Hand

        icon = <Label> {
            draw_text: {
                color: (NORD_AURORA_RED)
                text_style: (THEME_FONT_REGULAR) { font_size: 14.0 }
            }
            text: ""
        }

        time = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_CODE) { font_size: 12.0 }
            }
            text: "25:00"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PomodoroModule {
    #[deref]
    view: View,

    #[rust(25 * 60)]
    remaining_seconds: u32,

    #[rust]
    is_running: bool,

    #[rust]
    is_break: bool,
}

impl Widget for PomodoroModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(_) => {
                self.is_running = !self.is_running;
                cx.widget_action(
                    self.widget_uid(),
                    &HeapLiveIdPath::default(),
                    PomodoroAction::Toggle,
                );
            }
            _ => {}
        }
    }
}

impl PomodoroModule {
    pub fn set_time(&mut self, cx: &mut Cx, seconds: u32) {
        self.remaining_seconds = seconds;
        let minutes = seconds / 60;
        let secs = seconds % 60;
        let time_str = format!("{:02}:{:02}", minutes, secs);
        self.view.label(ids!(time)).set_text(cx, &time_str);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum PomodoroAction {
    None,
    Toggle,
    Reset,
}
