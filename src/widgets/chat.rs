use crate::services::cef::BrowserView;
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window, div};

pub struct ChatWidget {
    browser: Entity<BrowserView>,
}

impl ChatWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let browser = cx.new(|cx| BrowserView::new("https://gemini.google.com/app", 600, 1440, cx));
        Self { browser }
    }
}

impl gpui::Render for ChatWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.browser.clone())
    }
}
