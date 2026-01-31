use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub MprisModule = {{MprisModule}} {
        width: Fit, height: Fill
        flow: Row
        align: {x: 0.5, y: 0.5}
        spacing: 8
        padding: {left: 8, right: 8}

        visible: false

        prev_btn = <Button> {
            width: 24, height: 24
            draw_bg: { color: #0000 }
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 14.0 }
            }
            text: "󰒮"
        }

        play_pause_btn = <Button> {
            width: 24, height: 24
            draw_bg: { color: #0000 }
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
            }
            text: "󰐊"
        }

        next_btn = <Button> {
            width: 24, height: 24
            draw_bg: { color: #0000 }
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 14.0 }
            }
            text: "󰒭"
        }

        info = <View> {
            width: Fit, height: Fit
            flow: Down
            spacing: 2

            title = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: ""
            }

            artist = <Label> {
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
pub struct MprisModule {
    #[deref]
    view: View,

    #[rust]
    is_playing: bool,

    #[rust]
    title: String,

    #[rust]
    artist: String,
}

impl Widget for MprisModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl MprisModule {
    pub fn set_playing(&mut self, cx: &mut Cx, playing: bool) {
        self.is_playing = playing;
        let icon = if playing { "󰏤" } else { "󰐊" };
        self.view.button(ids!(play_pause_btn)).set_text(cx, icon);
    }

    pub fn set_track(&mut self, cx: &mut Cx, title: &str, artist: &str) {
        self.title = title.to_string();
        self.artist = artist.to_string();
        self.view.label(ids!(info.title)).set_text(cx, title);
        self.view.label(ids!(info.artist)).set_text(cx, artist);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum MprisAction {
    None,
    Play,
    Pause,
    Previous,
    Next,
}
