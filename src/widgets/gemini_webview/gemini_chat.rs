use gpui::*;
use gpui::prelude::*;
use crate::theme::*;
use crate::components::WebView;

pub struct GeminiChat {
    pub focus_handle: FocusHandle,
    webview: Entity<WebView>,
}

impl GeminiChat {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let webview = cx.new(|cx| {
            let builder = wry::WebViewBuilder::new();

            #[cfg(target_os = "linux")]
            let webview = {
                use gtk::prelude::*;
                use wry::WebViewBuilderExtUnix;

                let fixed = gtk::Fixed::builder().build();
                fixed.show_all();
                builder.build_gtk(&fixed).unwrap()
            };

            #[cfg(not(target_os = "linux"))]
            let webview = {
                use raw_window_handle::HasWindowHandle;
                let window_handle = window.window_handle().expect("No window handle");
                builder.build_as_child(&window_handle).unwrap()
            };

            WebView::new(webview, window, cx)
        });

        webview.update(cx, |view, _| {
            view.load_url("https://gemini.google.com/app");
        });

        Self {
            focus_handle: cx.focus_handle(),
            webview,
        }
    }

    pub fn reload(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.webview.update(cx, |view, _| {
            let _ = view.inner().evaluate_script("location.reload();");
        });
    }

    pub fn on_close(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.webview.update(cx, |view, _| {
            view.hide();
        });
        window.remove_window();
    }
}

impl Focusable for GeminiChat {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GeminiChat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(POLAR1))
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .w_full()
                    .h(px(56.0))
                    .bg(rgb(POLAR0))
                    .border_b_1()
                    .border_color(rgb(POLAR3))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(SNOW0))
                                    .child("Gemini Chat"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_lg()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::reload),
                                    )
                                    .child("↻"),
                            )
                            .child(
                                div()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_lg()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::on_close),
                                    )
                                    .child("×"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .child(self.webview.clone()),
            )
    }
}
