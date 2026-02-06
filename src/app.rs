use makepad_widgets::*;

use crate::{AUDIO_SERVICE, CAPSLOCK_SERVICE, CLIPBOARD_SERVICE, HYPRLAND_SERVICE, DBUS_LAUNCHER_SERVICE, DBUS_TASKER_SERVICE, APPLICATIONS_SERVICE};
use makepad_widgets::{LayerShellConfig, LayerShellLayer, LayerShellAnchor, LayerShellKeyboardInteractivity};
use crate::widgets::osd::{OSD, OSDAction, OSDWidgetRefExt};
use crate::widgets::launcher::{Launcher, LauncherAction, LauncherWidgetRefExt};
use crate::widgets::tasker::{Tasker, TaskerAction, TaskerWidgetRefExt};
use std::sync::Arc;
use parking_lot::RwLock;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    use crate::widgets::panel::*;
    use crate::widgets::launcher::*;
    use crate::widgets::osd::*;
    use crate::widgets::tasker::*;

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(3440, 68)},
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

            launcher = <Launcher> {}
        }

        tasker_window: <Window> {
            window: {inner_size: vec2(800, 600)},
            pass: {clear_color: #0000},

            tasker = <Tasker> {}
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
    #[live]
    tasker_window: WidgetRef,
    #[rust]
    launcher_visible: bool,
    #[rust]
    launcher_toggle_requested: Arc<RwLock<bool>>,
    #[rust]
    tasker_visible: bool,
    #[rust]
    tasker_toggle_requested: Arc<RwLock<bool>>,
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
        let _ = &*DBUS_TASKER_SERVICE;
        let _ = &*APPLICATIONS_SERVICE;

        if let Some(mut window) = self.ui.borrow_mut::<Window>() {
            window.set_layer_shell(cx, LayerShellConfig {
                layer: LayerShellLayer::Top,
                anchor: LayerShellAnchor::TOP | LayerShellAnchor::LEFT | LayerShellAnchor::RIGHT,
                exclusive_zone: Some(68),
                namespace: "nwidgets-panel".to_string(),
                keyboard_interactivity: LayerShellKeyboardInteractivity::None,
                margin: (0, 0, 0, 0),
                input_region: Some((0, 0, 3440, 68)),
            });
        }

        if let Some(mut window) = self.osd_window.borrow_mut::<Window>() {
            window.set_layer_shell(cx, LayerShellConfig {
                layer: LayerShellLayer::Overlay,
                anchor: LayerShellAnchor::BOTTOM,
                exclusive_zone: None,
                namespace: "nwidgets-osd".to_string(),
                keyboard_interactivity: LayerShellKeyboardInteractivity::None,
                margin: (0, 0, 100, 0),
                input_region: None,
            });
        }

        if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
            self.set_window_hidden(cx, &mut window, "nwidgets-launcher");
        }

        if let Some(mut window) = self.tasker_window.borrow_mut::<Window>() {
            self.set_window_hidden(cx, &mut window, "nwidgets-tasker");
        }

        let launcher_toggle_requested = self.launcher_toggle_requested.clone();

        DBUS_LAUNCHER_SERVICE.on_toggle(move || {
            let mut toggle = launcher_toggle_requested.write();
            *toggle = true;
        });

        let tasker_toggle_requested = self.tasker_toggle_requested.clone();

        DBUS_TASKER_SERVICE.on_toggle(move || {
            ::log::info!("DBus tasker toggle callback triggered");
            let mut toggle = tasker_toggle_requested.write();
            *toggle = true;
        });

        self.timer = cx.start_timeout(0.1);
    }

    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
    }
}

impl App {
    fn set_window_hidden(&self, cx: &mut Cx, window: &mut Window, namespace: &str) {
        window.set_layer_shell(cx, LayerShellConfig {
            layer: LayerShellLayer::Background,
            anchor: LayerShellAnchor::NONE,
            exclusive_zone: None,
            namespace: namespace.to_string(),
            keyboard_interactivity: LayerShellKeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
            input_region: None,
        });
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if self.timer.is_event(event).is_some() {
            if *self.launcher_toggle_requested.read() {
                *self.launcher_toggle_requested.write() = false;
                self.launcher_visible = !self.launcher_visible;

                    if self.launcher_visible {
                    if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                        launcher.show(cx);
                    }

                    if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                        launcher.set_text_input_focus(cx);
                    }
                } else {
                    if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                        launcher.hide(cx);
                    }
                }
            }

            if *self.tasker_toggle_requested.read() {
                *self.tasker_toggle_requested.write() = false;
                self.tasker_visible = !self.tasker_visible;
                
                ::log::info!("Tasker toggle: visible={}", self.tasker_visible);

                if self.tasker_visible {
                    if let Some(mut tasker) = self.tasker_window.tasker(ids!(tasker)).borrow_mut() {
                        ::log::info!("Showing tasker window");
                        tasker.show(cx);
                    } else {
                        ::log::error!("Failed to get tasker widget");
                    }
                } else {
                    if let Some(mut tasker) = self.tasker_window.tasker(ids!(tasker)).borrow_mut() {
                        ::log::info!("Hiding tasker window");
                        tasker.hide(cx);
                    }
                }
            }

            let current_state = AUDIO_SERVICE.state();

            if let Some(last_state) = &self.last_audio_state {
                if current_state.sink_volume != last_state.sink_volume
                    || current_state.sink_muted != last_state.sink_muted {
                    if let Some(mut osd) = self.osd_window.osd(ids!(osd)).borrow_mut() {
                        let volume = current_state.sink_volume as f32 / 100.0;
                        osd.show_volume(cx, volume, current_state.sink_muted);
                    }
                }
            }

            let capslock_state = CAPSLOCK_SERVICE.is_enabled();
            if self.last_capslock_state != Some(capslock_state) {
                if let Some(mut osd) = self.osd_window.osd(ids!(osd)).borrow_mut() {
                    osd.show_capslock(cx, capslock_state);
                }

                self.last_capslock_state = Some(capslock_state);
            }

            let clipboard_content = CLIPBOARD_SERVICE.get_last_content();
            if !clipboard_content.is_empty() && clipboard_content != self.last_clipboard_content {
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
                    LauncherAction::Shown => {
                        if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
                            window.set_layer_shell(cx, LayerShellConfig {
                                layer: LayerShellLayer::Overlay,
                                anchor: LayerShellAnchor::NONE,
                                exclusive_zone: None,
                                namespace: "nwidgets-launcher".to_string(),
                                keyboard_interactivity: LayerShellKeyboardInteractivity::Exclusive,
                                margin: (0, 0, 0, 0),
                                input_region: Some((0, 0, 700, 500)),
                            });
                        }
                        self.launcher_window.redraw(cx);
                        cx.redraw_all();
                    }
                    LauncherAction::Hidden => {
                        if let Some(mut window) = self.launcher_window.borrow_mut::<Window>() {
                            self.set_window_hidden(cx, &mut window, "nwidgets-launcher");
                        }
                        self.launcher_visible = false;
                        self.launcher_window.redraw(cx);
                        cx.redraw_all();
                    }
                    LauncherAction::Close => {
                        if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                            launcher.hide(cx);
                        }
                    }
                    LauncherAction::Launch(id) => {
                        if id.starts_with("calc:") {
                        } else if id.starts_with("ps:") {
                        } else {
                            if let Err(e) = APPLICATIONS_SERVICE.launch(&id) {
                                ::log::error!("Failed to launch application {}: {}", id, e);
                            }
                        }

                        if let Some(mut launcher) = self.launcher_window.launcher(ids!(launcher)).borrow_mut() {
                            launcher.hide(cx);
                        }
                    }
                    LauncherAction::QueryChanged(_query) => {
                    }
                    _ => {}
                }
                
                match action.as_widget_action().cast::<OSDAction>() {
                    OSDAction::Shown => {
                        if let Some(mut window) = self.osd_window.borrow_mut::<Window>() {
                            window.set_layer_shell(cx, LayerShellConfig {
                                layer: LayerShellLayer::Overlay,
                                anchor: LayerShellAnchor::BOTTOM,
                                exclusive_zone: None,
                                namespace: "nwidgets-osd".to_string(),
                                keyboard_interactivity: LayerShellKeyboardInteractivity::None,
                                margin: (0, 0, 100, 0),
                                input_region: Some((0, 0, 300, 80)),
                            });
                        }
                        self.osd_window.redraw(cx);
                        cx.redraw_all();
                    }
                    OSDAction::Hidden => {
                        if let Some(mut window) = self.osd_window.borrow_mut::<Window>() {
                            self.set_window_hidden(cx, &mut window, "nwidgets-osd");
                        }
                        self.osd_window.redraw(cx);
                        cx.redraw_all();
                    }
                    _ => {}
                }

                match action.as_widget_action().cast::<TaskerAction>() {
                    TaskerAction::Shown => {
                        if let Some(mut window) = self.tasker_window.borrow_mut::<Window>() {
                            window.set_layer_shell(cx, LayerShellConfig {
                                layer: LayerShellLayer::Overlay,
                                anchor: LayerShellAnchor::NONE,
                                exclusive_zone: None,
                                namespace: "nwidgets-tasker".to_string(),
                                keyboard_interactivity: LayerShellKeyboardInteractivity::Exclusive,
                                margin: (0, 0, 0, 0),
                                input_region: Some((0, 0, 800, 600)),
                            });
                        }
                        self.tasker_window.redraw(cx);
                        cx.redraw_all();
                    }
                    TaskerAction::Hidden => {
                        if let Some(mut window) = self.tasker_window.borrow_mut::<Window>() {
                            self.set_window_hidden(cx, &mut window, "nwidgets-tasker");
                        }
                        self.tasker_visible = false;
                        self.tasker_window.redraw(cx);
                        cx.redraw_all();
                    }
                    TaskerAction::Close => {
                        if let Some(mut tasker) = self.tasker_window.tasker(ids!(tasker)).borrow_mut() {
                            tasker.hide(cx);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.osd_window.handle_event(cx, event, &mut Scope::empty());
        self.launcher_window.handle_event(cx, event, &mut Scope::empty());
        self.tasker_window.handle_event(cx, event, &mut Scope::empty());
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui: WidgetRef::default(),
            osd_window: WidgetRef::default(),
            launcher_window: WidgetRef::default(),
            tasker_window: WidgetRef::default(),
            launcher_visible: false,
            launcher_toggle_requested: Arc::new(RwLock::new(false)),
            tasker_visible: false,
            tasker_toggle_requested: Arc::new(RwLock::new(false)),
            last_audio_state: None,
            last_capslock_state: None,
            last_clipboard_content: String::new(),
            timer: Timer::default(),
        }
    }
}
