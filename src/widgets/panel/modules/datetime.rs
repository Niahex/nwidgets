use makepad_widgets::*;
use chrono::Local;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub DateTimeModule = {{DateTimeModule}} {
        width: Fit, height: Fill
        flow: Down
        align: {x: 0.5, y: 0.5}
        spacing: 0
        padding: {left: 8, right: 8}

        time = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_BOLD) { font_size: 12.0 }
            }
            text: "00:00"
        }

        date = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_MUTE)
                text_style: (THEME_FONT_REGULAR) { font_size: 10.0 }
            }
            text: "Jan 01"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DateTimeModule {
    #[deref]
    view: View,

    #[rust]
    timer: Timer,
}

impl Widget for DateTimeModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.update_time(cx);
            self.timer = cx.start_timeout(1.0);
        }

        if let Event::Startup = event {
            self.update_time(cx);
            self.timer = cx.start_timeout(1.0);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl DateTimeModule {
    fn update_time(&mut self, cx: &mut Cx) {
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%b %d").to_string();

        self.view.label(ids!(time)).set_text(cx, &time_str);
        self.view.label(ids!(date)).set_text(cx, &date_str);
    }
}
