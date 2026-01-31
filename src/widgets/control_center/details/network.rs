use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NetworkDetails = {{NetworkDetails}} {
        width: Fill, height: Fill
        flow: Down
        padding: 16
        spacing: 12

        header = <View> {
            width: Fill, height: Fit
            flow: Row
            align: {x: 0.0, y: 0.5}

            title = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_BOLD) { font_size: 14.0 }
                }
                text: "Network"
            }
        }

        wifi_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            wifi_label = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: "WiFi"
            }

            wifi_status = <View> {
                width: Fill, height: 48
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                icon = <Label> {
                    draw_text: {
                        color: (THEME_COLOR_TEXT_DEFAULT)
                        text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                    }
                    text: "󰤨"
                }

                info = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 2

                    ssid = <Label> {
                        draw_text: {
                            color: (THEME_COLOR_TEXT_DEFAULT)
                            text_style: (THEME_FONT_REGULAR) { font_size: 12.0 }
                        }
                        text: "Not connected"
                    }

                    details = <Label> {
                        draw_text: {
                            color: (THEME_COLOR_TEXT_MUTE)
                            text_style: (THEME_FONT_REGULAR) { font_size: 10.0 }
                        }
                        text: ""
                    }
                }
            }
        }

        ethernet_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            ethernet_label = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: "Ethernet"
            }

            ethernet_status = <View> {
                width: Fill, height: 48
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                icon = <Label> {
                    draw_text: {
                        color: (THEME_COLOR_TEXT_DEFAULT)
                        text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                    }
                    text: "󰈀"
                }

                info = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 2

                    status = <Label> {
                        draw_text: {
                            color: (THEME_COLOR_TEXT_DEFAULT)
                            text_style: (THEME_FONT_REGULAR) { font_size: 12.0 }
                        }
                        text: "Not connected"
                    }
                }
            }
        }

        vpn_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            vpn_label = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: "VPN"
            }

            vpn_status = <View> {
                width: Fill, height: 48
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                icon = <Label> {
                    draw_text: {
                        color: (THEME_COLOR_TEXT_DEFAULT)
                        text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                    }
                    text: "󰦝"
                }

                info = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 2

                    status = <Label> {
                        draw_text: {
                            color: (THEME_COLOR_TEXT_DEFAULT)
                            text_style: (THEME_FONT_REGULAR) { font_size: 12.0 }
                        }
                        text: "Not connected"
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct NetworkDetails {
    #[deref]
    view: View,
}

impl Widget for NetworkDetails {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}
