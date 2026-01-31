use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub SinkModule = {{SinkModule}} {
        width: 32, height: 32
        align: {x: 0.5, y: 0.5}
        cursor: Hand

        icon = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
            }
            text: "󰕾"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SinkModule {
    #[deref]
    view: View,

    #[rust(0.5)]
    volume: f32,

    #[rust]
    is_muted: bool,
}

impl Widget for SinkModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerScroll(se) => {
                let delta = if se.scroll.y > 0.0 { 0.05 } else { -0.05 };
                self.volume = (self.volume + delta).max(0.0).min(1.0);
                self.update_icon(cx);

                cx.widget_action(
                    self.widget_uid(),
                    &HeapLiveIdPath::default(),
                    SinkAction::VolumeChanged(self.volume),
                );
            }
            Hit::FingerDown(_) => {
                self.is_muted = !self.is_muted;
                self.update_icon(cx);

                cx.widget_action(
                    self.widget_uid(),
                    &HeapLiveIdPath::default(),
                    SinkAction::MuteToggled(self.is_muted),
                );
            }
            _ => {}
        }
    }
}

impl SinkModule {
    pub fn set_volume(&mut self, cx: &mut Cx, volume: f32, muted: bool) {
        self.volume = volume;
        self.is_muted = muted;
        self.update_icon(cx);
    }

    fn update_icon(&mut self, cx: &mut Cx) {
        let icon = if self.is_muted {
            "󰖁"
        } else if self.volume > 0.66 {
            "󰕾"
        } else if self.volume > 0.33 {
            "󰖀"
        } else if self.volume > 0.0 {
            "󰕿"
        } else {
            "󰝟"
        };

        self.view.label(ids!(icon)).set_text(cx, icon);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum SinkAction {
    None,
    VolumeChanged(f32),
    MuteToggled(bool),
}
