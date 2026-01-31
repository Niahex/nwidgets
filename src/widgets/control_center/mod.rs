use makepad_widgets::*;

pub mod details;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    QuickActionButton = <Button> {
        width: 80, height: 80

        draw_bg: {
            color: (NORD_POLAR_2)
            radius: 12.0
        }

        flow: Down
        align: {x: 0.5, y: 0.5}
        spacing: 8
    }

    pub ControlCenter = {{ControlCenter}} {
        width: 400, height: Fill

        show_bg: true
        draw_bg: {
            color: (NORD_POLAR_0)
        }

        flow: Down
        padding: 16
        spacing: 16

        visible: false

        header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 0.0, y: 0.5}

            title = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_BOLD) { font_size: 16.0 }
                }
                text: "Control Center"
            }
        }

        quick_actions = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 12

            wifi_btn = <QuickActionButton> {
                text: "󰤨\nWiFi"
            }

            bluetooth_btn = <QuickActionButton> {
                text: "󰂯\nBluetooth"
            }

            dnd_btn = <QuickActionButton> {
                text: "󰍶\nDND"
            }

            night_btn = <QuickActionButton> {
                text: "󰖔\nNight"
            }
        }

        audio_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            audio_label = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: "Audio"
            }

            volume_slider = <View> {
                width: Fill, height: 40
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                icon = <Label> {
                    draw_text: {
                        color: (THEME_COLOR_TEXT_DEFAULT)
                        text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                    }
                    text: "󰕾"
                }

                slider = <Slider> {
                    width: Fill, height: 24
                }
            }

            mic_slider = <View> {
                width: Fill, height: 40
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                icon = <Label> {
                    draw_text: {
                        color: (THEME_COLOR_TEXT_DEFAULT)
                        text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                    }
                    text: "󰍬"
                }

                slider = <Slider> {
                    width: Fill, height: 24
                }
            }
        }

        system_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            system_label = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: "System"
            }

            monitors = <View> {
                width: Fill, height: Fit
                flow: Right
                spacing: 16
                align: {x: 0.5, y: 0.5}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ControlCenter {
    #[deref]
    view: View,

    #[rust]
    is_visible: bool,
}

impl Widget for ControlCenter {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl ControlCenter {
    pub fn toggle(&mut self, cx: &mut Cx) {
        self.is_visible = !self.is_visible;
        self.view.apply_over(cx, live! { visible: (self.is_visible) });
        self.view.redraw(cx);
    }

    pub fn show(&mut self, cx: &mut Cx) {
        self.is_visible = true;
        self.view.apply_over(cx, live! { visible: true });
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.is_visible = false;
        self.view.apply_over(cx, live! { visible: false });
        self.view.redraw(cx);
    }
}

pub fn register_live_design(cx: &mut Cx) {
    details::live_design(cx);
}
