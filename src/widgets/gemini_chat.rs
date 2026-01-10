use crate::services::cef::create_browser;
use cef::{Browser, ImplBrowser, ImplBrowserHost};
use cef_dll_sys::cef_mouse_button_type_t;
use gpui::{
    div, img, px, rgb, AsyncApp, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, RenderImage, ScrollWheelEvent,
    Styled, WeakEntity, Window,
};
use image::{Frame, ImageBuffer, Rgba};
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::sync::Arc;

pub struct GeminiChatWidget {
    browser: Option<Browser>,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
}

impl GeminiChatWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        eprintln!("GeminiChatWidget::new() called!");
        
        let pixels = Arc::new(Mutex::new(Vec::new()));
        let width = Arc::new(Mutex::new(600));
        let height = Arc::new(Mutex::new(1440));

        let pixels_clone = pixels.clone();
        let width_clone = width.clone();
        let height_clone = height.clone();

        let repaint_callback = Arc::new(move || {
            // Repaint callback - the refresh loop handles redraws
        });

        eprintln!("Creating CEF browser...");
        let browser = create_browser(
            "https://gemini.google.com/app".to_string(),
            pixels_clone,
            width_clone,
            height_clone,
            repaint_callback,
        );
        eprintln!("CEF browser created!");

        // Tell CEF to start rendering
        if let Some(host) = browser.host() {
            eprintln!("Calling was_resized() to trigger initial render...");
            host.was_resized();
        }

        // Refresh loop at 60 FPS
        cx.spawn(|view: WeakEntity<GeminiChatWidget>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let view = view.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(16))
                        .await;
                    let _ = view.update(&mut cx, |_, cx| cx.notify());
                }
            }
        })
        .detach();

        GeminiChatWidget {
            browser: Some(browser),
            pixels,
            width,
            height,
        }
    }
}

impl gpui::Render for GeminiChatWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let w = *self.width.lock();
        let h = *self.height.lock();
        let pixels = self.pixels.lock();

        let browser = self.browser.clone();

        if w > 0 && h > 0 && !pixels.is_empty() {
            if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, pixels.clone())
            {
                let frame = Frame::new(buffer);
                let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

                let browser_move = browser.clone();
                let browser_down = browser.clone();
                let browser_up = browser.clone();
                let browser_scroll = browser.clone();

                return div()
                    .size_full()
                    .on_mouse_move(move |event: &MouseMoveEvent, _window, _cx| {
                        if let Some(browser) = &browser_move {
                            if let Some(host) = browser.host() {
                                let pos = event.position;
                                let mouse_event = cef::MouseEvent {
                                    x: Into::<f32>::into(pos.x) as i32,
                                    y: Into::<f32>::into(pos.y) as i32,
                                    modifiers: 0,
                                };
                                host.send_mouse_move_event(Some(&mouse_event), 0);
                            }
                        }
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, _window, _cx| {
                            if let Some(browser) = &browser_down {
                                if let Some(host) = browser.host() {
                                    let pos = event.position;
                                    let mouse_event = cef::MouseEvent {
                                        x: Into::<f32>::into(pos.x) as i32,
                                        y: Into::<f32>::into(pos.y) as i32,
                                        modifiers: 0,
                                    };
                                    host.send_mouse_click_event(
                                        Some(&mouse_event),
                                        cef_mouse_button_type_t::MBT_LEFT.into(),
                                        0,
                                        1,
                                    );
                                }
                            }
                        },
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        move |event: &MouseUpEvent, _window, _cx| {
                            if let Some(browser) = &browser_up {
                                if let Some(host) = browser.host() {
                                    let pos = event.position;
                                    let mouse_event = cef::MouseEvent {
                                        x: Into::<f32>::into(pos.x) as i32,
                                        y: Into::<f32>::into(pos.y) as i32,
                                        modifiers: 0,
                                    };
                                    host.send_mouse_click_event(
                                        Some(&mouse_event),
                                        cef_mouse_button_type_t::MBT_LEFT.into(),
                                        1,
                                        1,
                                    );
                                }
                            }
                        },
                    )
                    .on_scroll_wheel(move |event: &ScrollWheelEvent, _window, _cx| {
                        if let Some(browser) = &browser_scroll {
                            if let Some(host) = browser.host() {
                                let pos = event.position;
                                let mouse_event = cef::MouseEvent {
                                    x: Into::<f32>::into(pos.x) as i32,
                                    y: Into::<f32>::into(pos.y) as i32,
                                    modifiers: 0,
                                };
                                let delta = event.delta.pixel_delta(px(1.0));
                                host.send_mouse_wheel_event(
                                    Some(&mouse_event),
                                    Into::<f32>::into(delta.x) as i32,
                                    Into::<f32>::into(delta.y) as i32,
                                );
                            }
                        }
                    })
                    .child(
                        img(Arc::new(render_image))
                            .w_full()
                            .h_full(),
                    )
                    .into_any_element();
            }
        }

        div()
            .flex()
            .size_full()
            .bg(rgb(0x2e3440))
            .text_color(rgb(0xd8dee9))
            .items_center()
            .justify_center()
            .child(format!("Gemini Chat ({}x{}) - Loading...", w, h))
            .into_any_element()
    }
}
