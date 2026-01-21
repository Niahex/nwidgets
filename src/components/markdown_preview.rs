use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

const REPARSE_DEBOUNCE: Duration = Duration::from_millis(200);

pub struct MarkdownPreview {
    content: SharedString,
    parsed_html: SharedString,
    parse_task: Option<Task<()>>,
}

impl MarkdownPreview {
    pub fn new(content: impl Into<SharedString>) -> Self {
        let content = content.into();
        Self {
            content: content.clone(),
            parsed_html: Self::parse_markdown_sync(&content),
            parse_task: None,
        }
    }

    pub fn update_content(&mut self, content: impl Into<SharedString>, cx: &mut Context<Self>) {
        let new_content = content.into();
        if self.content == new_content {
            return;
        }
        
        self.content = new_content.clone();
        self.parse_task = None;
        
        let content = new_content.clone();
        self.parse_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(REPARSE_DEBOUNCE).await;
            
            let parsed = cx.background_spawn(async move {
                Self::parse_markdown_sync(&content)
            }).await;
            
            _ = this.update(cx, |this, cx| {
                this.parsed_html = parsed;
                cx.notify();
            });
        }));
    }

    fn parse_markdown_sync(content: &str) -> SharedString {
        let mut html = String::new();
        
        for line in content.lines() {
            if line.starts_with("# ") {
                html.push_str(&format!("<h1>{}</h1>", &line[2..]));
            } else if line.starts_with("## ") {
                html.push_str(&format!("<h2>{}</h2>", &line[3..]));
            } else if line.starts_with("### ") {
                html.push_str(&format!("<h3>{}</h3>", &line[4..]));
            } else if line.starts_with("- ") {
                html.push_str(&format!("<li>{}</li>", &line[2..]));
            } else if !line.is_empty() {
                html.push_str(&format!("<p>{}</p>", line));
            }
        }
        
        html.into()
    }
}

impl Render for MarkdownPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(cx.theme().background())
            .p_4()
            .overflow_hidden()
            .child(
                div()
                    .text_color(cx.theme().text)
                    .child(self.parsed_html.clone())
            )
    }
}
