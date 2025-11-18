use crate::modules::{
    ActiveWindowModule, BluetoothService, BluetoothState, DateTimeModule, PomodoroModule,
    SystemTrayService, TrayItem, VolumeModule, WorkspaceModule,
};
use crate::services::{HyprlandService, PipeWireService};
use crate::theme::*;
use gpui::{div, prelude::*, rgb, Context, Window};
use std::time::Duration;

pub struct Panel {
    // Modules
    active_window_module: ActiveWindowModule,
    workspace_module: WorkspaceModule,
    pomodoro_module: PomodoroModule,
    volume_module: VolumeModule,
    datetime_module: DateTimeModule,

    // State
    tray_items: Vec<TrayItem>,
    bluetooth_state: BluetoothState,
}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (workspaces, active_workspace) = HyprlandService::get_hyprland_data();
        let active_window = HyprlandService::get_active_window();
        let pipewire_service = PipeWireService::new();
        let initial_volume = pipewire_service.get_volume();

        let panel = Self {
            active_window_module: ActiveWindowModule::new(active_window.clone()),
            workspace_module: WorkspaceModule::new(workspaces.clone(), active_workspace),
            pomodoro_module: PomodoroModule::new(),
            volume_module: VolumeModule::new(initial_volume),
            datetime_module: DateTimeModule::new(),
            tray_items: Vec::new(),
            bluetooth_state: BluetoothState {
                powered: false,
                connected_devices: 0,
            },
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

        // Initialize system tray
        cx.spawn(async move |this, cx| {
            println!("[PANEL] 🔔 Initializing system tray...");
            let mut tray_service = SystemTrayService::new();

            match tray_service.start_monitoring().await {
                Ok(items) => {
                    println!("[PANEL] 🔔 Found {} tray items", items.len());
                    let _ = this.update(cx, |panel, cx| {
                        panel.tray_items = items;
                        cx.notify();
                    });
                }
                Err(e) => {
                    println!("[PANEL] ❌ Failed to initialize systray: {:?}", e);
                }
            }
        })
        .detach();

        // Monitor Bluetooth state
        cx.spawn(async move |this, cx| {
            let receiver = BluetoothService::start_monitoring();

            loop {
                match receiver.try_recv() {
                    Ok(state) => {
                        let _ = this.update(cx, |panel, cx| {
                            if panel.bluetooth_state.powered != state.powered
                                || panel.bluetooth_state.connected_devices
                                    != state.connected_devices
                            {
                                println!(
                                    "[PANEL] 🔵 Bluetooth: powered={}, devices={}",
                                    state.powered, state.connected_devices
                                );
                                panel.bluetooth_state = state;
                                cx.notify();
                            }
                        });
                    }
                    Err(_) => {
                        gpui::Timer::after(Duration::from_millis(100)).await;
                    }
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
            PomodoroState::Idle => ("🍅", POLAR3),
            PomodoroState::Work | PomodoroState::WorkPaused => ("🍅", RED),
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => ("☕", YELLOW),
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => ("🌴", GREEN),
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
        let (bt_icon, bt_color) = if !self.bluetooth_state.powered {
            ("󰂯", RED) // Off - red
        } else if self.bluetooth_state.connected_devices > 0 {
            ("󰂱", FROST1) // Connected - blue
        } else {
            ("󰂲", SNOW0) // On but not connected - white
        };

        let mut bt_widget = div()
            .w_12()
            .h_8()
            .bg(rgb(POLAR2))
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(bt_color))
            .text_base()
            .cursor_pointer()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|_this, _event, _window, cx| {
                    cx.spawn(async move |_this, cx| {
                        match BluetoothService::toggle_power().await {
                            Ok(new_state) => {
                                println!("[PANEL] 🔵 Bluetooth toggled to: {}", new_state);
                            }
                            Err(e) => {
                                println!("[PANEL] ❌ Failed to toggle Bluetooth: {:?}", e);
                            }
                        }
                        let _ = cx;
                    })
                    .detach();
                }),
            )
            .child(bt_icon);

        if self.bluetooth_state.connected_devices > 0 {
            bt_widget = bt_widget.child(
                div()
                    .text_xs()
                    .ml_0p5()
                    .child(format!("{}", self.bluetooth_state.connected_devices)),
            );
        }

        bt_widget
    }

    fn render_systray(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .children(self.tray_items.iter().map(|item| {
                div()
                    .w_8()
                    .h_8()
                    .bg(rgb(POLAR2))
                    .rounded_sm()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .child(if !item.icon_name.is_empty() {
                        item.title.chars().next().unwrap_or('?').to_string()
                    } else {
                        "•".to_string()
                    })
            }))
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
            // Section gauche (fenêtre active + pomodoro)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .when_some(self.active_window_module.render(), |this, element| {
                        this.child(element)
                    })
                    .child(self.render_pomodoro(cx)),
            )
            // Section centrale (workspaces)
            .child(self.workspace_module.render())
            // Section droite (systray + bluetooth + volume + horloge)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(self.render_systray())
                    .child(self.render_bluetooth(cx))
                    .child(self.volume_module.render())
                    .child(self.datetime_module.render()),
            )
    }
}
