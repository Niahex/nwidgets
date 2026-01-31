use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub ActiveWindowModule = {{ActiveWindowModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.0, y: 0.5}
        spacing: 8

        icon = <View> {
            width: 24, height: 24
            align: {x: 0.5, y: 0.5}
        }

        info = <View> {
            width: Fit, height: Fit
            flow: Down
            spacing: 2

            title = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_REGULAR) { font_size: 12.0 }
                }
                text: "No window"
            }

            class = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 10.0 }
                }
                text: ""
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ActiveWindowModule {
    #[deref]
    view: View,

    #[rust]
    window_title: String,

    #[rust]
    window_class: String,
}

impl Widget for ActiveWindowModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl ActiveWindowModule {
    pub fn set_window_info(&mut self, cx: &mut Cx, title: &str, class: &str) {
        self.window_title = title.to_string();
        self.window_class = class.to_string();
        self.view.label(ids!(info.title)).set_text(cx, title);
        self.view.label(ids!(info.class)).set_text(cx, class);
    }
}
