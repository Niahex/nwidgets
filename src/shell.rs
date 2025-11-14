use gpui::{Context, Window, div, prelude::*, rgb, px, AnyElement};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::modules::{Notification, NotificationService};

#[derive(Debug, Deserialize, Clone)]
struct Workspace {
    id: i32,
    name: String,
    monitor: String,
    windows: i32,
    hasfullscreen: bool,
    lastwindow: String,
    lastwindowtitle: String,
}

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
}

impl Shell {
    pub fn new_background(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Background,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
        }
    }

    pub fn new_panel(_cx: &mut Context<Self>) -> Self {
        let (workspaces, active_workspace) = Self::get_hyprland_data();
        Self {
            mode: ShellMode::Panel,
            notifications: Vec::new(),
            workspaces,
            active_workspace,
        }
    }

    pub fn new_notifications(cx: &mut Context<Self>) -> Self {
        let (service, mut receiver) = NotificationService::new();
        service.start_dbus_server();
        
        let shell = Self {
            mode: ShellMode::Notifications,
            notifications: Vec::new(),
            workspaces: Vec::new(),
            active_workspace: 1,
        };

        // Réception des notifications
        cx.spawn(async move |this, cx| {
            while let Some(notification) = receiver.recv().await {
                let _ = this.update(cx, |shell, cx| {
                    shell.notifications.push(notification);
                    shell.notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    shell.notifications.truncate(10);
                    cx.notify();
                });
            }
        }).detach();

        // Timer pour nettoyer les notifications expirées
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let _ = this.update(cx, |shell, cx| {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let old_count = shell.notifications.len();
                    shell.notifications.retain(|n| now - n.timestamp < 5);
                    if shell.notifications.len() != old_count {
                        cx.notify();
                    }
                });
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

    fn get_hyprland_data() -> (Vec<Workspace>, i32) {
        let workspaces = Command::new("hyprctl")
            .args(&["workspaces", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Vec<Workspace>>(&json_str)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })
            .unwrap_or_default();

        let active_workspace = Command::new("hyprctl")
            .args(&["activeworkspace", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Workspace>(&json_str)
                    .map(|ws| ws.id)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })
            .unwrap_or(1);

        (workspaces, active_workspace)
    }

    fn get_volume_level(&self) -> u8 {
        Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .and_then(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(volume_str) = output_str.strip_prefix("Volume: ") {
                    if let Ok(volume_float) = volume_str.trim().parse::<f32>() {
                        return Ok((volume_float * 100.0) as u8);
                    }
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse error"))
            })
            .unwrap_or(50)
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
        let volume = self.get_volume_level();

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
                    .children(
                        self.workspaces.iter().take(5).map(|ws| {
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
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
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
                            .child(if volume == 0 { "🔇" } else if volume < 50 { "🔉" } else { "🔊" })
                            .child(format!("{}%", volume))
                    )
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Schedule re-render to update volume and time
        let entity = cx.entity_id();
        cx.defer(move |cx| {
            cx.notify(entity);
        });

        match self.mode {
            ShellMode::Background => self.render_background(),
            ShellMode::Panel => self.render_panel(),
            ShellMode::Notifications => self.render_notifications(),
        }
    }
}
