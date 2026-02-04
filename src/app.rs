use makepad_widgets::*;

use crate::{AUDIO_SERVICE, CAPSLOCK_SERVICE, CLIPBOARD_SERVICE, HYPRLAND_SERVICE, DBUS_LAUNCHER_SERVICE, APPLICATIONS_SERVICE};
use crate::widgets::osd::{OSD, OSDWidgetRefExt};
use crate::widgets::launcher::{Launcher, LauncherAction, LauncherWidgetRefExt};
use std::sync::Arc;
use parking_lot::RwLock;

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
            window: {inner_size: vec2(300, 80)},
            pass: {clear_color: #0000},

            <View> {
                width: Fill, height: Fill
                align: {x: 0.5, y: 0.5}

                osd = <OSD> {}
            }
        }

        launcher_window: <Window> {
            window: {inner_size: vec2(700, 500)},
            pass: {clear_color: #0000},

            <View> {
                width: Fill, height: Fill
                align: {x: 0.5, y: 0.5}

                launcher = <Launcher> {}
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
    #[live]
    launcher_window: WidgetRef,
    #[rust]
    layer_shell_configured: bool,
    #[rust]
    osd_layer_shell_configured: bool,
    #[rust]
    launcher_layer_shell_configured: bool,
    #[rust]
    launcher_visible: bool,
    #[rust]
    launcher_toggle_requested: Arc<RwLock<bool>>,
    #[rust]
    last_audio_state: Option<crate::services::media::audio::AudioState>,
    #[rust]
    last_capslock_state: Option<bool>,
    #[rust]
    last_clipboard_content: String,
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
        let _ = &*CAPSLOCK_SERVICE;
        let _ = &*DBUS_LAUNCHER_SERVICE;
        let _ = &*APPLICATIONS_SERVICE;

        let panel_config = LayerShellConfig {
            layer: LayerShellLayer::Top,
            anchor: LayerShellAnchor::TOP | LayerShellAnchor::LEFT | LayerShellAnchor::RIGHT,
            exclusive_zone: Some(68),
            namespace: "nwidgets-panel".to_string(),
            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
        };

        if let Some(mut window) = self.ui.borrow_mut::<Window>() {
            window.set_layer_shell(cx, panel_config);
            self.layer_shell_configured = true;
        }

        let osd_config = LayerShellConfig {
            layer: LayerShellLayer::Top,
            anchor: LayerShellAnchor::BOTTOM,
            exclusive_zone: None,
            namespace: "nwidgets-osd".to_string(),
            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
            margin: (0, 0, 100, 0),
        };

        if let Some(mut window) = self.osd_window.borrow_mut::<Window>() {
            window.set_layer_shell(cx, osd_config);
            self.osd_layer_shell_configured = true;
        }

        let launcher_config = LayerShellConfig {
            layer: LayerShellLayer::Overlay,
            anchor: LayerShellAnchor::NONE,
            exclusive_zone: None,
            namespace: "nwidgets-launcher".to_string(),
            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
        };

        if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
            window.set_layer_shell(cx, launcher_config);
            self.launcher_layer_shell_configured = true;
        }

        let launcher_toggle_requested = self.launcher_toggle_requested.clone();
        
        DBUS_LAUNCHER_SERVICE.on_toggle(move || {
            let mut toggle = launcher_toggle_requested.write();
            *toggle = true;
            ::log::info!("Launcher toggle requested via D-Bus");
        });
        
        self.timer = cx.start_timeout(0.1);
    }

    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if self.timer.is_event(event).is_some() {
            if *self.launcher_toggle_requested.read() {
                *self.launcher_toggle_requested.write() = false;
                self.launcher_visible = !self.launcher_visible;
                
                if self.launcher_visible {
                    ::log::info!("Showing launcher");
                    if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                        launcher.show(cx);
                    }
                    if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
                        window.set_layer_shell(cx, LayerShellConfig {
                            layer: LayerShellLayer::Overlay,
                            anchor: LayerShellAnchor::NONE,
                            exclusive_zone: None,
                            namespace: "nwidgets-launcher".to_string(),
                            keyboard_interactivity: LayerShellKeyboardInteractivity::Exclusive,
                            margin: (0, 0, 0, 0),
                        });
                    }
                    self.launcher_window.redraw(cx);
                    cx.redraw_all();
                } else {
                    ::log::info!("Hiding launcher");
                    if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                        launcher.hide(cx);
                    }
                    if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
                        window.set_layer_shell(cx, LayerShellConfig {
                            layer: LayerShellLayer::Overlay,
                            anchor: LayerShellAnchor::NONE,
                            exclusive_zone: None,
                            namespace: "nwidgets-launcher".to_string(),
                            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
                            margin: (0, 0, 0, 0),
                        });
                    }
                    self.launcher_window.redraw(cx);
                    cx.redraw_all();
                }
            }
            
            let current_state = AUDIO_SERVICE.state();

            if let Some(last_state) = &self.last_audio_state {
                if current_state.sink_volume != last_state.sink_volume
                    || current_state.sink_muted != last_state.sink_muted {
                    ::log::info!("OSD: Volume changed to {}% (muted: {})", current_state.sink_volume, current_state.sink_muted);

                    if let Some(mut osd) = self.osd_window.osd(ids!(osd)).borrow_mut() {
                        let volume = current_state.sink_volume as f32 / 100.0;
                        ::log::info!("OSD: Calling show_volume");
                        osd.show_volume(cx, volume, current_state.sink_muted);
                    } else {
                        ::log::warn!("OSD: Failed to borrow OSD widget");
                    }
                }
            }

            let capslock_state = CAPSLOCK_SERVICE.is_enabled();
            if self.last_capslock_state != Some(capslock_state) {
                ::log::info!("OSD: CapsLock changed to {}", capslock_state);

                if let Some(mut osd) = self.osd_window.osd(ids!(osd)).borrow_mut() {
                    osd.show_capslock(cx, capslock_state);
                }

                self.last_capslock_state = Some(capslock_state);
            }

            let clipboard_content = CLIPBOARD_SERVICE.get_last_content();
            if !clipboard_content.is_empty() && clipboard_content != self.last_clipboard_content {
                ::log::info!("OSD: Clipboard changed: {} bytes", clipboard_content.len());

                if let Some(mut osd) = self.osd_window.osd(ids!(osd)).borrow_mut() {
                    osd.show_clipboard(cx, &clipboard_content);
                }

                self.last_clipboard_content = clipboard_content;
            }

            self.last_audio_state = Some(current_state);
            self.timer = cx.start_timeout(0.1);
        }

        if let Event::Actions(actions) = event {
            for action in actions {
                match action.as_widget_action().cast::<LauncherAction>() {
                    LauncherAction::Close => {
                        self.launcher_visible = false;
                        if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                            launcher.hide(cx);
                        }
                    }
                    LauncherAction::Launch(id) => {
                        ::log::info!("Launching: {}", id);
                        
                        if id.starts_with("calc:") {
                            ::log::info!("Calculator result: {}", id);
                        } else if id.starts_with("ps:") {
                            ::log::info!("Process manager: {}", id);
                        } else {
                            if let Err(e) = APPLICATIONS_SERVICE.launch(&id) {
                                ::log::error!("Failed to launch application {}: {}", id, e);
                            } else {
                                ::log::info!("Successfully launched application: {}", id);
                            }
                        }
                        
                        self.launcher_visible = false;
                        if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                            launcher.hide(cx);
                        }
                    }
                    LauncherAction::QueryChanged(query) => {
                        ::log::info!("Query changed: {}", query);
                    }
                    _ => {}
                }
            }
        }

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.osd_window.handle_event(cx, event, &mut Scope::empty());
        self.launcher_window.handle_event(cx, event, &mut Scope::empty());
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui: WidgetRef::default(),
            osd_window: WidgetRef::default(),
            launcher_window: WidgetRef::default(),
            layer_shell_configured: false,
            osd_layer_shell_configured: false,
            launcher_layer_shell_configured: false,
            launcher_visible: false,
            launcher_toggle_requested: Arc::new(RwLock::new(false)),
            last_audio_state: None,
            last_capslock_state: None,
            last_clipboard_content: String::new(),
            timer: Timer::default(),
        }
    }
}
