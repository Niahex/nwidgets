use crate::services::launcher::{process::ProcessInfo, state::ApplicationInfo};
use crate::theme::Theme;
use gpui::{div, img, prelude::*};

#[derive(Clone)]
pub enum SearchResult {
    Application(ApplicationInfo),
    Calculation(String),
    Process(ProcessInfo),
    Clipboard(String),
}

pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    theme: Theme,
}

impl SearchResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            theme: Theme::nord_dark(),
        }
    }

    pub fn with_theme(mut self, theme: &Theme) -> Self {
        self.theme = theme.clone();
        self
    }

    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn move_selection_up(&mut self) {
        if !self.results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.results.is_empty() && self.selected_index + 1 < self.results.len() {
            self.selected_index += 1;
            let visible_items = 10;
            if self.selected_index >= self.scroll_offset + visible_items {
                self.scroll_offset = self.selected_index - visible_items + 1;
            }
        }
    }

    pub fn get_selected(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    pub fn render(&self) -> impl IntoElement {
        let visible_items = 10;
        let theme = self.theme.clone();
        let selected_index = self.selected_index;

        div().flex().flex_col().mt_2().children(
            self.results
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(visible_items)
                .map(|(original_index, result)| {
                    let mut item = div()
                        .flex()
                        .items_center()
                        .p_2()
                        .text_color(theme.text_muted)
                        .rounded_md()
                        .hover(|style| style.bg(theme.overlay));

                    if original_index == selected_index {
                        item = item.bg(theme.accent.opacity(0.2)).text_color(theme.accent);
                    }

                    match result {
                        SearchResult::Application(app) => item.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(if let Some(icon_path) = &app.icon_path {
                                    div()
                                        .size_6()
                                        .child(img(std::path::PathBuf::from(icon_path)).size_6())
                                } else {
                                    div().size_6().bg(theme.accent_alt).rounded_sm()
                                })
                                .child(app.name.clone()),
                        ),
                        SearchResult::Calculation(calc_result) => item.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .size_6()
                                        .bg(theme.green)
                                        .rounded_sm()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child("="),
                                )
                                .child(format!("= {calc_result}")),
                        ),
                        SearchResult::Process(process) => item.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .size_6()
                                        .bg(theme.red)
                                        .rounded_sm()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child("⚡"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(format!("{} ({})", process.name, process.pid))
                                        .child(div().text_xs().text_color(theme.accent).child(
                                            format!(
                                                "CPU: {:.1}% | RAM: {:.1}MB",
                                                process.cpu_usage, process.memory_mb
                                            ),
                                        )),
                                ),
                        ),
                        SearchResult::Clipboard(content) => {
                            let preview = if content.len() > 60 {
                                format!("{}...", &content[..60])
                            } else {
                                content.clone()
                            };
                            item.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .size_6()
                                            .bg(theme.accent)
                                            .rounded_sm()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child("📋"),
                                    )
                                    .child(preview),
                            )
                        }
                    }
                }),
        )
    }
}
