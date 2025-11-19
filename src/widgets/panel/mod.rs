use crate::modules::{
    ActiveWindowModule, BluetoothModule, DateTimeModule, DictationModule, PomodoroModule,
    SystrayModule, VolumeModule, WorkspaceModule,
};
use crate::services::{HotkeyEvent, HotkeyService, HyprlandService, PipeWireService};
use crate::theme::*;
use gpui::{div, prelude::*, rgb, Context, Window};
use std::time::Duration;

pub struct Panel {
    // Modules uniquement - pas de services !
    active_window_module: ActiveWindowModule,
    workspace_module: WorkspaceModule,
    pomodoro_module: PomodoroModule,
    volume_module: VolumeModule,
    datetime_module: DateTimeModule,
    bluetooth_module: BluetoothModule,
    systray_module: SystrayModule,
    dictation_module: DictationModule,
}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (workspaces, active_workspace) = HyprlandService::get_hyprland_data();
        let active_window = HyprlandService::get_active_window();
        let pipewire_service = PipeWireService::new();
        let initial_volume = pipewire_service.get_volume();

        // Initialize dictation module
        let mut dictation_module = DictationModule::new();
        if let Err(e) = dictation_module.initialize_default() {
            eprintln!("[PANEL] Failed to initialize dictation: {}", e);
            eprintln!("[PANEL] Dictation will not be available");
        }

        let panel = Self {
            active_window_module: ActiveWindowModule::new(active_window.clone()),
            workspace_module: WorkspaceModule::new(workspaces.clone(), active_workspace),
            pomodoro_module: PomodoroModule::new(),
            volume_module: VolumeModule::new(initial_volume),
            datetime_module: DateTimeModule::new(),
            bluetooth_module: BluetoothModule::new(),
            systray_module: SystrayModule::new(),
            dictation_module,
        };

        // Monitor volume changes using PipeWireService
        cx.spawn(async move |this, cx| {
            let pipewire_service = PipeWireService::new();
            let mut last_volume = pipewire_service.get_volume();

            loop {
                gpui::Timer::after(Duration::from_millis(500)).await;
                let new_volume = pipewire_service.get_volume();

                if new_volume != last_volume {
                    println!("[PANEL] 🔊 Volume: {}% -> {}%", last_volume, new_volume);
                    let _ = this.update(cx, |panel, cx| {
                        panel.volume_module.update(new_volume);
                        cx.notify();
                    });
                    last_volume = new_volume;
                }
            }
        })
        .detach();

        // Monitor Hyprland workspace and window changes
        cx.spawn(async move |this, cx| {
            let hyprland_rx = HyprlandService::start_monitoring();

            loop {
                match hyprland_rx.try_recv() {
                    Ok((workspaces, active_workspace, active_window)) => {
                        println!(
                            "[PANEL] 🖥️  Hyprland update -> workspace: {}, window: {:?}",
                            active_workspace,
                            active_window.as_ref().map(|w| &w.class)
                        );
                        let _ = this.update(cx, |panel, cx| {
                            panel.workspace_module.update(workspaces, active_workspace);
                            panel.active_window_module.update(active_window);
                            cx.notify();
                        });
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                    _ => {}
                }
                gpui::Timer::after(Duration::from_millis(100)).await;
            }
        })
        .detach();

        // Monitor Pomodoro timer
        cx.spawn(async move |this, cx| loop {
            gpui::Timer::after(Duration::from_millis(1000)).await;
            let _ = this.update(cx, |panel, cx| {
                panel.pomodoro_module.auto_transition();
                cx.notify();
            });
        })
        .detach();

        // Initialize system tray via module
        cx.spawn(async move |this, cx| {
            println!("[PANEL] 🔔 Initializing system tray...");

            match SystrayModule::start_monitoring().await {
                Ok(items) => {
                    println!("[PANEL] 🔔 Found {} tray items", items.len());
                    let _ = this.update(cx, |panel, cx| {
                        panel.systray_module.update(items);
                        cx.notify();
                    });
                }
                Err(e) => {
                    println!("[PANEL] ❌ Failed to initialize systray: {:?}", e);
                }
            }
        })
        .detach();

        // Monitor Bluetooth state via module
        cx.spawn(async move |this, cx| {
            let receiver = BluetoothModule::start_monitoring();

            loop {
                match receiver.try_recv() {
                    Ok(state) => {
                        let _ = this.update(cx, |panel, cx| {
                            panel.bluetooth_module.update(state);
                            cx.notify();
                        });
                    }
                    Err(_) => {
                        gpui::Timer::after(Duration::from_millis(100)).await;
                    }
                }
            }
        })
        .detach();

        // Monitor hotkey events (Super+Space for dictation)
        cx.spawn(async move |this, cx| {
            println!("[PANEL] Starting hotkey monitoring...");
            match HotkeyService::start() {
                Ok(hotkey_service) => {
                    println!("[PANEL] Hotkey service started - listening for Super+Space");
                    loop {
                        if let Some(event) = hotkey_service.poll_event() {
                            match event {
                                HotkeyEvent::DictationToggle => {
                                    let _ = this.update(cx, |panel, cx| {
                                        panel.dictation_module.toggle_recording();
                                        cx.notify();
                                    });
                                }
                            }
                        }
                        gpui::Timer::after(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    eprintln!("[PANEL] Failed to start hotkey service: {}", e);
                    eprintln!("[PANEL] Make sure you have permission to access /dev/input devices");
                    eprintln!("[PANEL] Run: sudo usermod -a -G input $USER");
                }
            }
        })
        .detach();

        panel
    }

    fn render_pomodoro(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::services::PomodoroState;

        let current_state = self.pomodoro_module.get_service_mut().get_state();
        let (pomodoro_icon, pomodoro_color) = match current_state {
            PomodoroState::Idle => ("", POLAR3), // nf-md-timer_outline
            PomodoroState::Work | PomodoroState::WorkPaused => ("", RED), // nf-md-timer
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => ("", YELLOW), // nf-md-coffee
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => ("󱁕", GREEN), // nf-md-beach
        };

        div()
            .w_12()
            .h_12()
            .bg(rgb(pomodoro_color))
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .text_color(rgb(POLAR0))
            .text_xs()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |panel, _event, _window, cx| {
                    match current_state {
                        PomodoroState::Idle => {
                            panel.pomodoro_module.get_service_mut().start_work();
                        }
                        PomodoroState::Work
                        | PomodoroState::ShortBreak
                        | PomodoroState::LongBreak => {
                            panel.pomodoro_module.get_service_mut().pause();
                        }
                        PomodoroState::WorkPaused
                        | PomodoroState::ShortBreakPaused
                        | PomodoroState::LongBreakPaused => {
                            panel.pomodoro_module.get_service_mut().resume();
                        }
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(move |panel, _event, _window, cx| {
                    panel.pomodoro_module.get_service_mut().reset();
                    cx.notify();
                }),
            )
            .child(pomodoro_icon)
            .child(self.pomodoro_module.get_service_mut().format_time())
    }

    fn render_bluetooth(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Le module gère tout : le rendu ET les événements
        self.bluetooth_module.render(cx)
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(POLAR1))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            // Section gauche (fenêtre active)
            .child(self.active_window_module.render())
            // Section centrale (workspaces)
            .child(self.workspace_module.render())
            // Section droite (dictation + pomodoro + systray + bluetooth + volume + horloge)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(self.dictation_module.render())
                    .child(self.render_pomodoro(cx))
                    .child(self.systray_module.render())
                    .child(self.render_bluetooth(cx))
                    .child(self.volume_module.render())
                    .child(self.datetime_module.render()),
            )
    }
}
