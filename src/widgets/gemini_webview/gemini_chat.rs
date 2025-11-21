use crate::components::WebView;
use crate::theme::*;
use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

pub struct GeminiChat {
    pub focus_handle: FocusHandle,
    webview: Entity<WebView>,
    #[cfg(target_os = "linux")]
    gtk_window: Rc<gtk::Window>,
}

impl GeminiChat {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        #[cfg(target_os = "linux")]
        let gtk_window = {
            use gtk::prelude::*;
            use gtk_layer_shell::{Edge, Layer, LayerShell};

            println!("[GEMINI] Creating GTK window with layer shell");
            let gtk_win = gtk::Window::new(gtk::WindowType::Toplevel);

            // Initialize layer shell
            gtk_win.init_layer_shell();
            gtk_win.set_layer(Layer::Overlay);

            // Set anchors to match GPUI window position (left side)
            gtk_win.set_anchor(Edge::Left, true);
            gtk_win.set_anchor(Edge::Top, true);
            gtk_win.set_anchor(Edge::Bottom, true);
            gtk_win.set_anchor(Edge::Right, false);

            // Set fixed width to match GPUI window
            gtk_win.set_size_request(500, -1);

            println!("[GEMINI] GTK Layer shell window configured");
            Rc::new(gtk_win)
        };

        let webview = cx.new(|cx| {
            println!("[GEMINI] Creating webview builder");
            let mut builder = wry::WebViewBuilder::new();

            // Enable devtools for debugging
            #[cfg(any(debug_assertions, feature = "inspector"))]
            {
                println!("[GEMINI] Enabling devtools");
                builder = builder.with_devtools(true);
            }

            #[cfg(target_os = "linux")]
            let webview = {
                use gtk::prelude::*;
                use wry::WebViewBuilderExtUnix;

                println!("[GEMINI] Creating GTK Fixed container");
                let fixed = gtk::Fixed::builder().build();

                // Add fixed to GTK window
                gtk_window.add(&fixed);
                gtk_window.show_all();

                println!("[GEMINI] Building webview with GTK");
                builder.build_gtk(&fixed).unwrap()
            };

            #[cfg(not(target_os = "linux"))]
            let webview = {
                use raw_window_handle::HasWindowHandle;
                let window_handle = window.window_handle().expect("No window handle");
                builder.build_as_child(&window_handle).unwrap()
            };

            println!("[GEMINI] Webview created successfully");
            WebView::new(webview, window, cx)
        });

        println!("[GEMINI] Loading URL: https://google.com");
        webview.update(cx, |view, _| {
            // Test with Google first to verify webview works
            view.load_url("https://google.com");
        });

        Self {
            focus_handle: cx.focus_handle(),
            webview,
            #[cfg(target_os = "linux")]
            gtk_window,
        }
    }

    pub fn reload(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        println!("[GEMINI] Reload button clicked");
        self.webview.update(cx, |view, _| {
            // Cycle through test URLs to debug
            let test_html = r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>WebView Test</title>
                    <style>
                        body {
                            font-family: Arial, sans-serif;
                            padding: 40px;
                            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            color: white;
                        }
                        h1 { font-size: 48px; }
                        button {
                            padding: 15px 30px;
                            font-size: 18px;
                            margin: 10px;
                            cursor: pointer;
                            background: white;
                            color: #667eea;
                            border: none;
                            border-radius: 8px;
                        }
                    </style>
                </head>
                <body>
                    <h1>WebView is Working! 🎉</h1>
                    <p>This HTML is loaded directly into the webview.</p>
                    <button onclick="alert('JavaScript works!')">Test JavaScript</button>
                    <button onclick="window.location.href='https://google.com'">Go to Google</button>
                    <button onclick="window.location.href='https://gemini.google.com/app'">Go to Gemini</button>
                </body>
                </html>
            "#;

            println!("[GEMINI] Loading test HTML");
            view.load_html(test_html);
        });
    }

    pub fn on_close(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.webview.update(cx, |view, _| {
            view.hide();
        });

        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            println!("[GEMINI] Closing GTK window");
            self.gtk_window.close();
        }

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
                        div().flex().items_center().gap_3().child(
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
            .child(div().flex_1().w_full().child(self.webview.clone()))
    }
}
