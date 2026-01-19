use crate::theme::Theme;
use gpui::{div, prelude::*, SharedString};

pub struct SearchInput {
    pub query: SharedString,
    pub placeholder: String,
    theme: Theme,
}

impl SearchInput {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            query: "".into(),
            placeholder: placeholder.into(),
            theme: Theme::nord_dark(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_query(&mut self, query: impl Into<SharedString>) {
        self.query = query.into();
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn render_with_handlers<F1, F2>(&self, _on_input: F1, _on_space: F2) -> impl IntoElement
    where
        F1: Fn(&str) + 'static,
        F2: Fn(&str) + 'static,
    {
        let query_text = self.query.to_string();
        let theme = self.theme.clone();
        let placeholder = self.placeholder.clone();

        div()
            .p_2()
            .bg(theme.surface)
            .rounded_md()
            .flex()
            .gap_1()
            .text_color(if query_text.is_empty() {
                theme.text_muted
            } else {
                theme.text
            })
            .child(if query_text.is_empty() {
                div().child(placeholder)
            } else if query_text.starts_with("ps") {
                self.render_command_input("ps", &query_text, theme.green)
            } else if query_text.starts_with("clip") {
                self.render_command_input("clip", &query_text, theme.accent)
            } else if query_text.starts_with('=') {
                self.render_command_input("=", &query_text, theme.accent_alt)
            } else {
                div().child(query_text.clone())
            })
    }

    fn render_command_input(&self, cmd: &str, query_text: &str, bg_color: gpui::Hsla) -> gpui::Div {
        let (command, rest) = if query_text.starts_with(cmd) && query_text.len() > cmd.len() {
            (
                cmd.to_string(),
                query_text.strip_prefix(cmd).unwrap_or("").to_string(),
            )
        } else if query_text == cmd {
            (cmd.to_string(), String::new())
        } else {
            (String::new(), query_text.to_string())
        };

        div()
            .flex()
            .gap_1()
            .child(
                div()
                    .px_1()
                    .bg(bg_color)
                    .text_color(self.theme.text)
                    .rounded_sm()
                    .child(command),
            )
            .child(rest)
    }
}
