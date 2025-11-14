use gpui::{Context, Window, div, prelude::*, rgb, rgba};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Panel {
    enabled: bool,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.enabled {
            return div().size_full();
        }

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
    }
}
