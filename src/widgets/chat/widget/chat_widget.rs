use crate::services::cef::BrowserView;
use crate::theme::ActiveTheme;
use crate::widgets::chat::service::{load_url, ChatService};
use crate::widgets::chat::types::ChatToggled;
use crate::widgets::chat::widget::styles::CSS;
use gpui::prelude::*;
use gpui::{
    div, Animation, AnimationExt, AppContext, Context, Entity, IntoElement, ParentElement, Styled,
    Window,
};

pub struct ChatWidget {
    browser: Entity<BrowserView>,
    chat_service: Entity<ChatService>,
}

impl ChatWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let url = load_url();

        let injection_script = format!(
            "const style=document.createElement('style');style.textContent=`{}`;document.head.appendChild(style);",
            CSS.replace('`', "\\`").replace("${ ", "\\${ ")
        );

        let browser = cx.new(|cx| BrowserView::new(&url, 600, 1370, Some(&injection_script), cx));
        let chat_service = ChatService::global(cx);

        browser.read(cx).set_hidden(true);

        let browser_clone = browser.clone();
        cx.subscribe(
            &chat_service,
            move |_this, service, _event: &ChatToggled, cx| {
                let visible = service.read(cx).visible;
                browser_clone.read(cx).set_hidden(!visible);
                cx.notify();
            },
        )
        .detach();

        Self {
            browser,
            chat_service,
        }
    }

    pub fn current_url(&self, cx: &gpui::App) -> Option<String> {
        self.browser.read(cx).current_url()
    }

    pub fn navigate(&self, url: &str, cx: &mut gpui::App) {
        self.browser.read(cx).navigate(url);
    }

    pub fn resize_browser(&self, width: u32, height: u32, cx: &gpui::App) {
        self.browser.read(cx).resize(width, height);
    }

    pub fn focus_input(&self, cx: &gpui::App) {
        let js = r#"
            const editor = document.querySelector('.ql-editor[contenteditable="true"]');
            if (editor) {
                editor.focus();
            }
        "#;
        self.browser.read(cx).execute_js(js);
    }
}

impl gpui::Render for ChatWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.chat_service.read(cx).visible;

        if !visible {
            return div().into_any_element();
        }

        let theme = cx.theme();

        div()
            .id("chat-root")
            .size_full()
            .occlude()
            .bg(theme.bg)
            .rounded(gpui::px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .child(self.browser.clone())
            .with_animation(
                "chat-fade-in",
                Animation::new(std::time::Duration::from_millis(150)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
