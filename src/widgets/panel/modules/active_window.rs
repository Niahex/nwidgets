use makepad_widgets::*;

use crate::HYPRLAND_SERVICE;

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
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "No window"
            }

            class = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
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

    #[rust]
    timer: Timer,
}

impl Widget for ActiveWindowModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        if let Event::Startup = event {
            ::log::info!("ActiveWindowModule: Startup event received");
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl ActiveWindowModule {
    fn sync_from_service(&mut self, cx: &mut Cx) {
        ::log::info!("ActiveWindowModule: sync_from_service called");
        let window = HYPRLAND_SERVICE.get_active_window();
        ::log::info!("ActiveWindowModule: got window {} - {}", window.class, window.title);
        
        if window.title != self.window_title || window.class != self.window_class {
            self.window_title = window.title.clone();
            self.window_class = window.class.clone();
            
            let title = if window.title.is_empty() {
                "No window"
            } else {
                &window.title
            };
            
            self.view.label(ids!(info.title)).set_text(cx, title);
            self.view.label(ids!(info.class)).set_text(cx, &window.class);
        }
    }
}
