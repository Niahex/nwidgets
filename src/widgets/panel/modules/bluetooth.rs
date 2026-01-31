use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub BluetoothModule = {{BluetoothModule}} {
        width: 32, height: 32
        align: {x: 0.5, y: 0.5}

        icon = <Label> {
            draw_text: {
                color: (THEME_COLOR_TEXT_DEFAULT)
                text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
            }
            text: "󰂯"
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BluetoothModule {
    #[deref]
    view: View,

    #[rust]
    is_connected: bool,

    #[rust]
    is_enabled: bool,
}

impl Widget for BluetoothModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl BluetoothModule {
    pub fn set_status(&mut self, cx: &mut Cx, enabled: bool, connected: bool) {
        self.is_enabled = enabled;
        self.is_connected = connected;

        let icon = if !enabled {
            "󰂲"
        } else if connected {
            "󰂱"
        } else {
            "󰂯"
        };

        self.view.label(ids!(icon)).set_text(cx, icon);
    }
}
