use crate::modules::{
    paint_cove_corner_clipped, CoveCornerConfig, CoveCornerPosition, Notification,
    NotificationService,
};
use crate::services::hyprland::Workspace;
use crate::services::{HyprlandService, PipeWireService, PomodoroService, PomodoroState};
use gpui::{canvas, div, prelude::*, px, rgb, AnyElement, Context, Hsla, Window};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Nord Dark palette
const NORD0: u32 = 0x2e3440;
const NORD1: u32 = 0x3b4252;
const NORD2: u32 = 0x434c5e;
const NORD3: u32 = 0x4c566a;
const NORD4: u32 = 0xd8dee9;
const NORD8: u32 = 0x88c0d0;
const NORD9: u32 = 0x81a1c1;
const NORD10: u32 = 0x5e81ac;
const NORD11: u32 = 0xbf616a;
const NORD13: u32 = 0xebcb8b;
const NORD14: u32 = 0xa3be8c;

pub enum ShellMode {
    Background,
    Panel,
    Notifications,
    Corner(CornerPosition),
}

#[derive(Clone, Copy)]
pub enum CornerPosition {
    BottomLeft,
    BottomRight,
}

pub struct Shell {
    mode: ShellMode,
    notifications: Vec<Notification>,
    workspaces: Vec<Workspace>,
    active_workspace: i32,
    volume: u8,
    pomodoro: PomodoroService,
}

impl Shell {
    pub fn new_background(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Background,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
            volume: 50,
            pomodoro: PomodoroService::new(),
        }
    }

    pub fn new_panel(cx: &mut Context<Self>) -> Self {
        let (workspaces, active_workspace) = HyprlandService::get_hyprland_data();
        let pipewire_service = PipeWireService::new();
        let initial_volume = pipewire_service.get_volume();

        let shell = Self {
            mode: ShellMode::Panel,
            notifications: Vec::new(),
            workspaces,
            active_workspace,
            volume: initial_volume,
            pomodoro: PomodoroService::new(),
        };

        // Monitor volume changes using PipeWireService
        cx.spawn(async move |this, cx| {
            let pipewire_service = PipeWireService::new();
            let mut last_volume = pipewire_service.get_volume();

            loop {
                gpui::Timer::after(Duration::from_millis(500)).await;
                let new_volume = pipewire_service.get_volume();

                if new_volume != last_volume {
                    println!("[SHELL] 🔊 Volume: {}% -> {}%", last_volume, new_volume);
                    let _ = this.update(cx, |shell, cx| {
                        shell.volume = new_volume;
                        cx.notify();
                    });
                    last_volume = new_volume;
                }
            }
        })
        .detach();

        // Monitor Hyprland workspace changes
        cx.spawn(async move |this, cx| {
            let hyprland_rx = HyprlandService::start_monitoring();

            loop {
                match hyprland_rx.try_recv() {
                    Ok((workspaces, active_workspace)) => {
                        println!("[SHELL] 🖥️  Workspace changed -> {}", active_workspace);
                        let _ = this.update(cx, |shell, cx| {
                            shell.workspaces = workspaces;
                            shell.active_workspace = active_workspace;
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
            let _ = this.update(cx, |shell, cx| {
                shell.pomodoro.auto_transition();
                cx.notify();
            });
        })
        .detach();

        shell
    }

    pub fn new_notifications(cx: &mut Context<Self>) -> Self {
        let (service, mut receiver) = NotificationService::new();
        service.start_dbus_server();

        let shell = Self {
            mode: ShellMode::Notifications,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
            volume: 50,
            pomodoro: PomodoroService::new(),
        };

        // Réception des notifications
        cx.spawn(async move |this, cx| {
            println!("[SHELL] 📢 Notification receiver task started");
            while let Some(notification) = receiver.recv().await {
                println!(
                    "[SHELL] 📢 Received notification from channel: {} - {}",
                    notification.summary, notification.body
                );
                let result = this.update(cx, |shell, cx| {
                    println!(
                        "[SHELL] 📢 Adding notification to shell (current count: {})",
                        shell.notifications.len()
                    );
                    shell.notifications.push(notification.clone());
                    shell
                        .notifications
                        .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    shell.notifications.truncate(10);
                    println!(
                        "[SHELL] 📢 Now have {} notifications, calling cx.notify()",
                        shell.notifications.len()
                    );
                    cx.notify();
                });
                if let Err(e) = result {
                    println!("[SHELL] ❌ Error updating shell with notification: {:?}", e);
                }
            }
            println!("[SHELL] ⚠️  Notification receiver channel closed!");
        })
        .detach();

        // Timer pour nettoyer les notifications expirées
        cx.spawn(async move |this, cx| {
            println!("[SHELL] 📢 Notification cleanup task started");
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let result = this.update(cx, |shell, cx| {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let old_count = shell.notifications.len();
                    shell.notifications.retain(|n| now - n.timestamp < 5);
                    if shell.notifications.len() != old_count {
                        println!(
                            "[SHELL] 📢 Cleaned up notifications: {} -> {}",
                            old_count,
                            shell.notifications.len()
                        );
                        cx.notify();
                    }
                });
                if let Err(e) = result {
                    println!("[SHELL] ❌ Error cleaning notifications: {:?}", e);
                }
            }
        })
        .detach();

        shell
    }

    pub fn new_corner(_cx: &mut Context<Self>, position: CornerPosition) -> Self {
        Self {
            mode: ShellMode::Corner(position),
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
            volume: 50,
            pomodoro: PomodoroService::new(),
        }
    }

    fn render_pomodoro(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (pomodoro_icon, pomodoro_color) = match self.pomodoro.get_state() {
            PomodoroState::Idle => ("🍅", NORD3),
            PomodoroState::Work | PomodoroState::WorkPaused => ("🍅", NORD11),
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => ("☕", NORD13),
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => ("🌴", NORD14),
        };

        let current_state = self.pomodoro.get_state();

        div()
            .w_12()
            .h_12()
            .bg(rgb(pomodoro_color))
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .text_color(rgb(NORD0))
            .text_xs()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |shell, _event, _window, cx| {
                    match current_state {
                        PomodoroState::Idle => {
                            shell.pomodoro.start_work();
                        }
                        PomodoroState::Work
                        | PomodoroState::ShortBreak
                        | PomodoroState::LongBreak => {
                            shell.pomodoro.pause();
                        }
                        PomodoroState::WorkPaused
                        | PomodoroState::ShortBreakPaused
                        | PomodoroState::LongBreakPaused => {
                            shell.pomodoro.resume();
                        }
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(move |shell, _event, _window, cx| {
                    shell.pomodoro.reset();
                    cx.notify();
                }),
            )
            .child(pomodoro_icon)
            .child(self.pomodoro.format_time())
            .into_any_element()
    }

    fn render_background(&self) -> AnyElement {
        // Background supprimé - simplement retourner un div vide
        div()
            .size_full()
            .into_any_element()
    }

    fn render_panel(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Add timezone offset for CET (UTC+1)
        let local_time = now + 3600;
        let hours = (local_time / 3600) % 24;
        let minutes = (local_time / 60) % 60;

        div()
            .size_full()
            .bg(rgb(NORD1))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            // Section gauche (pomodoro)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(self.render_pomodoro(cx)),
            )
            // Section centrale (workspaces)
            .child(div().flex().flex_row().items_center().gap_2().children({
                let mut sorted_workspaces = self.workspaces.clone();
                // Sort: 1-6 first, then others
                sorted_workspaces.sort_by(|a, b| match (a.id <= 6, b.id <= 6) {
                    (true, true) => a.id.cmp(&b.id),
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (false, false) => a.id.cmp(&b.id),
                });

                sorted_workspaces
                    .into_iter()
                    .take(8)
                    .map(|ws| {
                        let is_active = ws.id == self.active_workspace;
                        let bg_color = if is_active { rgb(NORD10) } else { rgb(NORD2) };
                        div()
                            .w_8()
                            .h_8()
                            .bg(bg_color)
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child(ws.id.to_string())
                    })
                    .collect::<Vec<_>>()
            }))
            // Section droite (volume + horloge)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child({
                        let volume_icon = if self.volume == 0 {
                            "🔇"
                        } else if self.volume < 50 {
                            "🔉"
                        } else {
                            "🔊"
                        };
                        div()
                            .w_16()
                            .h_8()
                            .bg(rgb(NORD14))
                            .rounded_md()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .text_color(rgb(NORD0))
                            .text_xs()
                            .child(volume_icon)
                            .child(format!("{}%", self.volume))
                    })
                    .child(
                        div()
                            .w_16()
                            .h_8()
                            .bg(rgb(NORD3))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_sm()
                            .child(format!("{:02}:{:02}", hours, minutes)),
                    ),
            )
            .into_any_element()
    }

    fn render_corner(&self, position: CornerPosition) -> AnyElement {
        // Crée un coin arrondi inversé (cove/concave) selon la formule mathématique:
        // Coin = (Carré S×S ∖ Disque(centre=(S/2,S/2), rayon=S/2)) ∩ Quadrant
        //
        // Le coin est créé en dessinant un cercle de la couleur du panel qui "mord"
        // le carré transparent, créant l'effet de découpe circulaire (cove).

        let cove_position = match position {
            CornerPosition::BottomLeft => CoveCornerPosition::TopRight,
            CornerPosition::BottomRight => CoveCornerPosition::TopLeft,
        };

        canvas(
            move |_bounds, _window, _cx| {
                // Prepaint: retourner la configuration du coin
                let panel_color: Hsla = rgb(NORD13).into();
                CoveCornerConfig::new(px(48.0), panel_color, cove_position)
            },
            move |bounds, config, window, cx| {
                // Paint: dessiner le coin cove avec clipping
                paint_cove_corner_clipped(window, cx, bounds, &config);
            },
        )
        .size_full()
        .into_any_element()
    }

    fn render_notifications(&self) -> AnyElement {
        let mut container = div().size_full().flex().flex_col().gap_3().p_4();

        for notification in &self.notifications {
            let border_color = match notification.urgency {
                2 => NORD11, // Critical - rouge
                1 => NORD13, // Normal - jaune
                _ => NORD8,  // Low - bleu
            };

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - notification.timestamp;

            let time_str = if elapsed < 60 {
                "now".to_string()
            } else if elapsed < 3600 {
                format!("{}m ago", elapsed / 60)
            } else {
                format!("{}h ago", elapsed / 3600)
            };

            let mut notif_div = div()
                .w_full()
                .bg(rgb(NORD2))
                .rounded_lg()
                .p_4()
                .border_l_4()
                .border_color(rgb(border_color))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_color(rgb(NORD4))
                                .text_sm()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(notification.summary.clone()),
                        )
                        .child(div().text_color(rgb(NORD3)).text_xs().child(time_str)),
                )
                .child(
                    div()
                        .text_color(rgb(NORD4))
                        .text_xs()
                        .child(notification.body.clone()),
                );

            if !notification.app_name.is_empty() {
                notif_div = notif_div.child(
                    div()
                        .text_color(rgb(NORD3))
                        .text_xs()
                        .child(format!("from {}", notification.app_name)),
                );
            }

            container = container.child(notif_div);
        }

        // Ne rien afficher si la liste est vide

        container.into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            ShellMode::Background => self.render_background(),
            ShellMode::Panel => self.render_panel(cx),
            ShellMode::Notifications => self.render_notifications(),
            ShellMode::Corner(position) => self.render_corner(position),
        }
    }
}
