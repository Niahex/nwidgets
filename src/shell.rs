use gpui::{Context, Window, div, prelude::*, rgb, rgba, px, AnyElement};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum ShellMode {
    Background,
    Panel,
}

pub struct Shell {
    mode: ShellMode,
}

impl Shell {
    pub fn new_background(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Background,
        }
    }

    pub fn new_panel(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Panel,
        }
    }

    fn render_background(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(0x1a1a1a)) // Dark background
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .child(
                // Wallpaper placeholder
                div()
                    .size_full()
                    .bg(rgb(0x2a2a2a))
            )
            .child(
                // Desktop clock placeholder (bottom right)
                div()
                    .absolute()
                    .bottom(px(20.))
                    .right(px(20.))
                    .p_4()
                    .bg(rgb(0x333333))
                    .rounded_md()
                    .text_color(rgb(0xffffff))
                    .child("12:34")
            )
            .into_any_element()
    }

    fn render_panel(&self) -> AnyElement {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;

        div()
            .size_full()
            .bg(rgba(0x1a1a1aaa))
            .border_b_1()
            .border_color(rgba(0x444444aa))
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .child(
                // Left side - App launcher / menu
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Menu")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Apps")
                    )
            )
            .child(
                // Center - Window title or workspace info
                div()
                    .text_color(rgb(0xffffff))
                    .text_sm()
                    .child("Workspace 1")
            )
            .child(
                // Right side - System tray, clock, etc.
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("🔊")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("🔋")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x444444aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child(format!("{:02}:{:02}", hours, minutes))
                    )
            )
            .into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            ShellMode::Background => self.render_background(),
            ShellMode::Panel => self.render_panel(),
        }
    }
}
