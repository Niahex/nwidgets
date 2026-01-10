use cef::{CefString, ImplBrowserHost};
use gpui::*;

pub struct FindBar {
    pub visible: bool,
    pub query: String,
}

impl FindBar {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.query.clear();
        }
    }

    pub fn handle_key(&mut self, key: &str, key_char: Option<&str>, modifiers: &Modifiers, host: &cef::BrowserHost) -> bool {
        if !self.visible {
            return false;
        }

        match key {
            "escape" => {
                self.visible = false;
                self.query.clear();
                host.stop_finding(1);
                return true;
            }
            "backspace" => {
                self.query.pop();
                self.find(host, false);
                return true;
            }
            "enter" | "down" => {
                self.find_next(host);
                return true;
            }
            "up" => {
                self.find_previous(host);
                return true;
            }
            _ => {
                if let Some(ch) = key_char.and_then(|s| s.chars().next()) {
                    if !modifiers.control && !modifiers.alt {
                        self.query.push(ch);
                        self.find(host, false);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn find(&self, host: &cef::BrowserHost, find_next: bool) {
        host.find(
            Some(&CefString::from(self.query.as_str())),
            1,
            0,
            if find_next { 1 } else { 0 },
        );
    }

    pub fn find_next(&self, host: &cef::BrowserHost) {
        host.find(Some(&CefString::from(self.query.as_str())), 1, 0, 1);
    }

    pub fn find_previous(&self, host: &cef::BrowserHost) {
        host.find(Some(&CefString::from(self.query.as_str())), 0, 0, 1);
    }

    pub fn close(&mut self, host: &cef::BrowserHost) {
        self.visible = false;
        self.query.clear();
        host.stop_finding(1);
    }

    pub fn render(&self, on_prev: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static, on_next: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static, on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
        let query = if self.query.is_empty() {
            "Type to search...".to_string()
        } else {
            self.query.clone()
        };
        div()
            .absolute()
            .bottom_0()
            .left_0()
            .right_0()
            .h(px(50.))
            .bg(rgb(0x3b4252))
            .border_t_1()
            .border_color(rgb(0x4c566a))
            .flex()
            .items_center()
            .px_4()
            .gap_2()
            .child(
                div()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2e3440))
                    .rounded_md()
                    .text_color(rgb(0xeceff4))
                    .child(query),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x4c566a))
                    .rounded_md()
                    .text_color(rgb(0xeceff4))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, on_prev)
                    .child("↑"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x4c566a))
                    .rounded_md()
                    .text_color(rgb(0xeceff4))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, on_next)
                    .child("↓"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x4c566a))
                    .rounded_md()
                    .text_color(rgb(0xeceff4))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, on_close)
                    .child("✕"),
            )
    }
}
