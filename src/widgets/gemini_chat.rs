use crate::services::cef::{CefNavigated, CefReady, CefService};
use gpui::prelude::*;
use gpui::*;

const GEMINI_URL: &str = "https://gemini.google.com/app";

pub struct GeminiChatWidget {
    pub focus_handle: FocusHandle,
    cef: Entity<CefService>,
    is_loading: bool,
}

impl GeminiChatWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let cef = CefService::global(cx);

        // Initialize CEF
        cef.update(cx, |service, cx| service.initialize(cx));

        cx.subscribe(&cef, |this, _, _event: &CefReady, cx| {
            this.is_loading = false;
            // Auto-load Gemini when ready
            this.load_gemini(cx);
            cx.notify();
        })
        .detach();

        cx.subscribe(&cef, |_this, _, _event: &CefNavigated, cx| {
            cx.notify();
        })
        .detach();

        Self {
            focus_handle: cx.focus_handle(),
            cef,
            is_loading: true,
        }
    }

    pub fn load_gemini(&mut self, cx: &mut Context<Self>) {
        self.is_loading = true;
        self.cef
            .update(cx, |cef, cx| cef.navigate(GEMINI_URL.to_string(), cx));
        cx.notify();
    }
}

impl Render for GeminiChatWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_ready = self.cef.read(cx).is_ready();
        let current_url = self.cef.read(cx).current_url();

        div()
            .id("gemini-chat-widget")
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(rgb(0x2d2d2d))
                    .border_b_1()
                    .border_color(rgb(0x3e3e3e))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w_3()
                                    .h_3()
                                    .rounded_full()
                                    .bg(if is_ready {
                                        rgb(0x4ade80)
                                    } else {
                                        rgb(0xfbbf24)
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("Gemini Chat"),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x3b82f6))
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x2563eb)))
                            .child("Reload"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(if self.is_loading {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .w_12()
                                    .h_12()
                                    .rounded_full()
                                    .border_4()
                                    .border_color(rgb(0x3b82f6)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9ca3af))
                                    .child("Loading Gemini..."),
                            )
                    } else if is_ready {
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("Gemini Chat Ready"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!(
                                        "URL: {}",
                                        current_url.unwrap_or_else(|| "None".to_string())
                                    )),
                            )
                            .child(
                                div()
                                    .mt_4()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0x374151))
                                    .text_sm()
                                    .text_color(rgb(0xd1d5db))
                                    .child("WebView integration coming soon..."),
                            )
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xef4444))
                                    .child("Not Ready"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9ca3af))
                                    .child("Click Reload to start"),
                            )
                    }),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x2d2d2d))
                    .border_t_1()
                    .border_color(rgb(0x3e3e3e))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("Gemini Chat Widget - Powered by CEF WebView"),
                    ),
            )
    }
}
