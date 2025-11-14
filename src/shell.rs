use gpui::{Context, Window, div, prelude::*, rgb, px, AnyElement};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::modules::{Notification, NotificationService};
use crate::services::{HyprlandService, PipeWireService};
use crate::services::hyprland::Workspace;

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
}

pub struct Shell {
    mode: ShellMode,
    notifications: Vec<Notification>,
    workspaces: Vec<Workspace>,
    active_workspace: i32,
    volume: u8,
}

impl Shell {
    pub fn new_background(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Background,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
            volume: 50,
        }
    }

    pub fn new_panel(cx: &mut Context<Self>) -> Self {
        println!("[SHELL] 🎨 Creating panel shell");
        let (workspaces, active_workspace) = HyprlandService::get_hyprland_data();
        let pipewire_service = PipeWireService::new();
        let initial_volume = pipewire_service.get_volume();
        println!("[SHELL] 📊 Initial state - volume: {}, active_workspace: {}, workspaces: {}",
            initial_volume, active_workspace, workspaces.len());

        let shell = Self {
            mode: ShellMode::Panel,
            notifications: Vec::new(),
            workspaces,
            active_workspace,
            volume: initial_volume,
        };

        // Subscribe to PipeWire volume changes
        cx.spawn(async move |this, cx| {
            let mut volume_rx = PipeWireService::start_monitoring();
            println!("[SHELL] 🔊 Subscribed to volume changes");

            while let Some((new_volume, _muted)) = volume_rx.recv().await {
                println!("[SHELL] 🔊 Received volume update: {}%", new_volume);
                let result = this.update(cx, |shell, cx| {
                    shell.volume = new_volume;
                    println!("[SHELL] 🔔 Calling cx.notify() for volume update");
                    cx.notify();
                });

                if let Err(e) = result {
                    println!("[SHELL] ❌ Error updating volume: {:?}", e);
                    break;
                }
            }
        }).detach();

        // Subscribe to Hyprland workspace changes
        cx.spawn(async move |this, cx| {
            let mut hyprland_rx = HyprlandService::start_monitoring();
            println!("[SHELL] 🖥️  Subscribed to Hyprland changes");

            while let Some((workspaces, active_workspace)) = hyprland_rx.recv().await {
                println!("[SHELL] 📡 Received workspace update - active: {}, count: {}",
                    active_workspace, workspaces.len());

                let result = this.update(cx, |shell, cx| {
                    shell.workspaces = workspaces;
                    shell.active_workspace = active_workspace;
                    println!("[SHELL] 🔔 Calling cx.notify() to trigger re-render");
                    cx.notify();
                });

                if let Err(e) = result {
                    println!("[SHELL] ❌ Error updating workspaces: {:?}", e);
                    break;
                }
            }
        }).detach();

        shell
    }

    pub fn new_notifications(cx: &mut Context<Self>) -> Self {
        println!("[SHELL] 📢 Creating notifications shell");
        let (service, mut receiver) = NotificationService::new();
        println!("[SHELL] 📢 NotificationService created, starting D-Bus server");
        service.start_dbus_server();

        let shell = Self {
            mode: ShellMode::Notifications,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
            volume: 50,
        };

        // Réception des notifications
        cx.spawn(async move |this, cx| {
            println!("[SHELL] 📢 Notification receiver task started");
            while let Some(notification) = receiver.recv().await {
                println!("[SHELL] 📢 Received notification from channel: {} - {}",
                    notification.summary, notification.body);
                let result = this.update(cx, |shell, cx| {
                    println!("[SHELL] 📢 Adding notification to shell (current count: {})",
                        shell.notifications.len());
                    shell.notifications.push(notification.clone());
                    shell.notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    shell.notifications.truncate(10);
                    println!("[SHELL] 📢 Now have {} notifications, calling cx.notify()",
                        shell.notifications.len());
                    cx.notify();
                });
                if let Err(e) = result {
                    println!("[SHELL] ❌ Error updating shell with notification: {:?}", e);
                }
            }
            println!("[SHELL] ⚠️  Notification receiver channel closed!");
        }).detach();

        // Timer pour nettoyer les notifications expirées
        cx.spawn(async move |this, cx| {
            println!("[SHELL] 📢 Notification cleanup task started");
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let result = this.update(cx, |shell, cx| {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let old_count = shell.notifications.len();
                    shell.notifications.retain(|n| now - n.timestamp < 5);
                    if shell.notifications.len() != old_count {
                        println!("[SHELL] 📢 Cleaned up notifications: {} -> {}",
                            old_count, shell.notifications.len());
                        cx.notify();
                    }
                });
                if let Err(e) = result {
                    println!("[SHELL] ❌ Error cleaning notifications: {:?}", e);
                }
            }
        }).detach();

        shell
    }

    fn render_background(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(NORD0))
            .child(
                div()
                    .size_full()
                    .bg(rgb(NORD1))
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(20.))
                    .right(px(20.))
                    .p_4()
                    .bg(rgb(NORD2))
                    .rounded_md()
                    .text_color(rgb(NORD4))
                    .child("12:34")
            )
            .into_any_element()
    }

    fn render_panel(&self) -> AnyElement {
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
            .border_r_1()
            .border_color(rgb(NORD3))
            .flex()
            .flex_col()
            .justify_between()
            .py_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_10()
                            .h_10()
                            .bg(rgb(NORD8))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("📱")
                    )
                    .child(
                        div()
                            .w_10()
                            .h_10()
                            .bg(rgb(NORD9))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🚀")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .children({
                        let mut sorted_workspaces = self.workspaces.clone();
                        // Sort: 1-6 first, then others
                        sorted_workspaces.sort_by(|a, b| {
                            match (a.id <= 6, b.id <= 6) {
                                (true, true) => a.id.cmp(&b.id),
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                (false, false) => a.id.cmp(&b.id),
                            }
                        });
                        
                        sorted_workspaces.into_iter().take(8).map(|ws| {
                            let is_active = ws.id == self.active_workspace;
                            let bg_color = if is_active { rgb(NORD10) } else { rgb(NORD2) };
                            println!("[SHELL] Rendering workspace {} - active: {} (current active: {}), color: {:?}",
                                ws.id, is_active, self.active_workspace, bg_color);
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
                        }).collect::<Vec<_>>()
                    })
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child({
                        let volume_icon = if self.volume == 0 { "🔇" } else if self.volume < 50 { "🔉" } else { "🔊" };
                        println!("[SHELL] Rendering volume widget: {}% - icon: {}", self.volume, volume_icon);
                        div()
                            .w_10()
                            .h_8()
                            .bg(rgb(NORD14))
                            .rounded_md()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_xs()
                            .child(volume_icon)
                            .child(format!("{}%", self.volume))
                    })
                    .child(
                        div()
                            .w_10()
                            .h_12()
                            .bg(rgb(NORD3))
                            .rounded_md()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child(format!("{:02}", hours))
                            .child(format!("{:02}", minutes))
                    )
            )
            .into_any_element()
    }

    fn render_notifications(&self) -> AnyElement {
        let mut container = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4();

        for notification in &self.notifications {
            let border_color = match notification.urgency {
                2 => NORD11, // Critical - rouge
                1 => NORD13, // Normal - jaune
                _ => NORD8,  // Low - bleu
            };

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - notification.timestamp;

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
                                .child(notification.summary.clone())
                        )
                        .child(
                            div()
                                .text_color(rgb(NORD3))
                                .text_xs()
                                .child(time_str)
                        )
                )
                .child(
                    div()
                        .text_color(rgb(NORD4))
                        .text_xs()
                        .child(notification.body.clone())
                );

            if !notification.app_name.is_empty() {
                notif_div = notif_div.child(
                    div()
                        .text_color(rgb(NORD3))
                        .text_xs()
                        .child(format!("from {}", notification.app_name))
                );
            }

            container = container.child(notif_div);
        }

        if self.notifications.is_empty() {
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .text_color(rgb(NORD3))
                    .text_sm()
                    .child("No notifications")
            );
        }

        container.into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            ShellMode::Background => {
                println!("[SHELL] Rendering background");
                self.render_background()
            },
            ShellMode::Panel => {
                println!("[SHELL] Rendering panel - volume: {}, active_workspace: {}, workspaces: {}", 
                    self.volume, self.active_workspace, self.workspaces.len());
                self.render_panel()
            },
            ShellMode::Notifications => {
                println!("[SHELL] Rendering notifications - count: {}", self.notifications.len());
                self.render_notifications()
            },
        }
    }
}
