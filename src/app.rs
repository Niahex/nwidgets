use makepad_widgets::*;

use crate::{AUDIO_SERVICE, CLIPBOARD_SERVICE, HYPRLAND_SERVICE};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    use crate::widgets::panel::*;
    use crate::widgets::launcher::*;
    use crate::widgets::osd::*;

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
        
        osd_window: <Window> {
            window: {inner_size: vec2(300, 80), position: vec2(810, 500)},
            pass: {clear_color: #0000},
            
            <View> {
                width: Fill, height: Fill
                align: {x: 0.5, y: 0.5}
                
                osd = <OSD> {}
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[live]
    osd_window: WidgetRef,
    #[rust]
    layer_shell_configured: bool,
    #[rust]
    last_audio_state: Option<crate::services::media::audio::AudioState>,
    #[rust]
    timer: Timer,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        let _ = &*HYPRLAND_SERVICE;
        let _ = &*AUDIO_SERVICE;
        let _ = &*CLIPBOARD_SERVICE;
        
        let config = LayerShellConfig {
            layer: LayerShellLayer::Top,
            anchor: LayerShellAnchor::TOP | LayerShellAnchor::LEFT | LayerShellAnchor::RIGHT,
            exclusive_zone: Some(68),
            namespace: "nwidgets-panel".to_string(),
            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
        };
        
        if let Some(mut window) = self.ui.borrow_mut::<Window>() {
            window.set_layer_shell(cx, config);
            self.layer_shell_configured = true;
        }
        
        self.timer = cx.start_timeout(0.1);
    }

    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if self.timer.is_event(event).is_some() {
            let current_state = AUDIO_SERVICE.state();
            
            if let Some(last_state) = &self.last_audio_state {
                if current_state.sink_volume != last_state.sink_volume 
                    || current_state.sink_muted != last_state.sink_muted {
                    if let Some(mut osd) = self.osd_window.view(ids!(osd)).borrow_mut::<crate::widgets::osd::OSD>() {
                        let volume = current_state.sink_volume as f32 / 100.0;
                        osd.show_volume(cx, volume, current_state.sink_muted);
                    }
                }
            }
            
            self.last_audio_state = Some(current_state);
            self.timer = cx.start_timeout(0.1);
        }
        
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.osd_window.handle_event(cx, event, &mut Scope::empty());
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui: WidgetRef::default(),
            osd_window: WidgetRef::default(),
            layer_shell_configured: false,
            last_audio_state: None,
            timer: Timer::default(),
        }
    }
}
