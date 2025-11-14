use gpui::{Context, Window, div, prelude::*, rgb, px, AnyElement};
use std::time::{SystemTime, UNIX_EPOCH};

// Nord Dark palette
const NORD0: u32 = 0x2e3440;  // Polar Night
const NORD1: u32 = 0x3b4252;
const NORD2: u32 = 0x434c5e;
const NORD3: u32 = 0x4c566a;
const NORD4: u32 = 0xd8dee9;  // Snow Storm
const NORD5: u32 = 0xe5e9f0;
const NORD6: u32 = 0xeceff4;
const NORD7: u32 = 0x8fbcbb;  // Frost
const NORD8: u32 = 0x88c0d0;
const NORD9: u32 = 0x81a1c1;
const NORD10: u32 = 0x5e81ac;
const NORD11: u32 = 0xbf616a; // Aurora
const NORD12: u32 = 0xd08770;
const NORD13: u32 = 0xebcb8b;
const NORD14: u32 = 0xa3be8c;
const NORD15: u32 = 0xb48ead;

pub enum ShellMode {
    Background,
    Panel,
    DrawerTop,
    DrawerBottom,
    DrawerRight,
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

    pub fn new_drawer_top(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::DrawerTop,
        }
    }

    pub fn new_drawer_bottom(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::DrawerBottom,
        }
    }

    pub fn new_drawer_right(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::DrawerRight,
        }
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
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;

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
                    .child(
                        div()
                            .w_8()
                            .h_8()
                            .bg(rgb(NORD10))
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child("1")
                    )
                    .child(
                        div()
                            .w_8()
                            .h_8()
                            .bg(rgb(NORD2))
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child("2")
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
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🔊")
                    )
                    .child(
                        div()
                            .w_10()
                            .h_8()
                            .bg(rgb(NORD13))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🔋")
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

    fn render_drawer_top(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(NORD1))
            .border_b_1()
            .border_color(rgb(NORD3))
            .into_any_element()
    }

    fn render_drawer_bottom(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(NORD1))
            .border_t_1()
            .border_color(rgb(NORD3))
            .into_any_element()
    }

    fn render_drawer_right(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(NORD1))
            .border_l_1()
            .border_color(rgb(NORD3))
            .into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            ShellMode::Background => self.render_background(),
            ShellMode::Panel => self.render_panel(),
            ShellMode::DrawerTop => self.render_drawer_top(),
            ShellMode::DrawerBottom => self.render_drawer_bottom(),
            ShellMode::DrawerRight => self.render_drawer_right(),
        }
    }
}
