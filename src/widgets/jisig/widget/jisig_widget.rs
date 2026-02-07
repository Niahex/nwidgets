use crate::services::cef::BrowserView;
use crate::theme::ActiveTheme;
use crate::widgets::jisig::service::JisigService;
use crate::widgets::jisig::types::JisigToggled;
use gpui::prelude::*;
use gpui::{
    div, Animation, AnimationExt, AppContext, Context, Entity, IntoElement, ParentElement, Styled,
    Window,
};

pub struct JisigWidget {
    browser: Entity<BrowserView>,
    jisig_service: Entity<JisigService>,
}

impl JisigWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let url = "http://localhost:8000/private";

        let browser = cx.new(|cx| BrowserView::new(url, 600, 1370, None, cx));
        let jisig_service = JisigService::global(cx);

        browser.read(cx).set_hidden(true);

        let browser_clone = browser.clone();
        cx.subscribe(
            &jisig_service,
            move |_this, service, _event: &JisigToggled, cx| {
                let visible = service.read(cx).visible;
                browser_clone.read(cx).set_hidden(!visible);
                cx.notify();
            },
        )
        .detach();

        Self {
            browser,
            jisig_service,
        }
    }

    pub fn resize_browser(&self, width: u32, height: u32, cx: &gpui::App) {
        self.browser.read(cx).resize(width, height);
    }
}

impl gpui::Render for JisigWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.jisig_service.read(cx).visible;

        if !visible {
            return div().into_any_element();
        }

        let theme = cx.theme();

        div()
            .id("jisig-root")
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
                "jisig-fade-in",
                Animation::new(std::time::Duration::from_millis(150)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
