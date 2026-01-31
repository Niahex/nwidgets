use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NetworkModule = {{NetworkModule}} {
        width: 32, height: 32
        align: {x: 0.5, y: 0.5}

        icon = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
            }
            text: "󰤨"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct NetworkModule {
    #[deref]
    view: View,

    #[rust]
    connection_type: NetworkType,

    #[rust]
    signal_strength: u8,
}

#[derive(Clone, Debug, Default)]
pub enum NetworkType {
    #[default]
    Disconnected,
    Wifi,
    Ethernet,
    Vpn,
}

impl Widget for NetworkModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl NetworkModule {
    pub fn set_status(&mut self, cx: &mut Cx, conn_type: NetworkType, strength: u8) {
        self.connection_type = conn_type.clone();
        self.signal_strength = strength;

        let icon = match conn_type {
            NetworkType::Disconnected => "󰤭",
            NetworkType::Ethernet => "󰈀",
            NetworkType::Vpn => "󰦝",
            NetworkType::Wifi => {
                if strength > 75 {
                    "󰤨"
                } else if strength > 50 {
                    "󰤥"
                } else if strength > 25 {
                    "󰤢"
                } else {
                    "󰤟"
                }
            }
        };

        self.view.label(ids!(icon)).set_text(cx, icon);
    }
}
