use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    use crate::widgets::panel::*;
    use crate::widgets::launcher::*;

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1920, 68)},
            pass: {clear_color: #0000},

            body = <NordView> {
                width: Fill, height: Fill
                flow: Down

                panel = <Panel> {}
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    layer_shell_configured: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        makepad_widgets::log!("nwidgets started (Makepad port)");
        
        if let Some(mut window) = self.ui.window(&[id!(ui)]).borrow_mut() {
            let config = LayerShellConfig {
                layer: LayerShellLayer::Top,
                anchor: LayerShellAnchor::TOP | LayerShellAnchor::LEFT | LayerShellAnchor::RIGHT,
                exclusive_zone: Some(68),
                namespace: "nwidgets-panel".to_string(),
                keyboard_interactivity: LayerShellKeyboardInteractivity::None,
                margin: (0, 0, 0, 0),
            };
            window.set_layer_shell(cx, config);
            self.layer_shell_configured = true;
            makepad_widgets::log!("Layer-shell configured for panel");
        }
    }

    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui: WidgetRef::default(),
            layer_shell_configured: false,
        }
    }
}
