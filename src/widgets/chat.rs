use crate::services::cef::{create_browser, CefService};
use cef::{Browser, BrowserHost, ImplBrowser, ImplBrowserHost};
use cef::sys::cef_mouse_button_type_t::{MBT_LEFT};
use gpui::{
    App, Context, Entity, IntoElement, Render, Styled, Window, rgb, 
    AppContext, ParentElement, RenderImage, AsyncApp, WeakEntity, InteractiveElement,
    MouseMoveEvent, MouseDownEvent, MouseUpEvent, ScrollWheelEvent, MouseButton,
    FocusHandle, px, Bounds, Pixels
};
use parking_lot::Mutex;
use std::sync::Arc;
use image::{Frame, ImageBuffer, Rgba};
use smallvec::SmallVec;

pub struct Chat {
    browser: Option<Browser>,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    focus_handle: FocusHandle,
}

impl Chat {
    pub fn new(cx: &mut App) -> Entity<Self> {
        if !cx.has_global::<CefService>() {
            eprintln!("CefService not initialized!");
        }

        cx.new(|cx| {
            let pixels = Arc::new(Mutex::new(Vec::new()));
            let width = Arc::new(Mutex::new(600));
            let height = Arc::new(Mutex::new(1000));

            let pixels_clone = pixels.clone();
            let width_clone = width.clone();
            let height_clone = height.clone();

            let repaint_callback = Arc::new(move || {});
            
            let browser = create_browser(
                "https://google.com".to_string(),
                pixels_clone,
                width_clone,
                height_clone,
                repaint_callback
            );

            let view_handle = cx.entity().clone();
            cx.spawn(|view: WeakEntity<Chat>, mut cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                let view = view.clone();
                async move {
                    loop {
                        cx.background_executor().timer(std::time::Duration::from_millis(16)).await;
                        let _ = view.update(&mut cx, |_, cx| cx.notify());
                    }
                }
            }).detach();

            Chat {
                browser: Some(browser),
                pixels,
                width,
                height,
                focus_handle: cx.focus_handle(),
            }
        })
    }

    fn get_host(&self) -> Option<BrowserHost> {
        self.browser.as_ref()?.host()
    }

    fn sync_size(&self, bounds: Bounds<Pixels>) {
        let new_w = bounds.size.width.to_f64() as u32;
        let new_h = bounds.size.height.to_f64() as u32;
        
        let mut w = self.width.lock();
        let mut h = self.height.lock();
        
        if *w != new_w || *h != new_h {
            *w = new_w;
            *h = new_h;
            if let Some(host) = self.get_host() {
                host.was_resized();
            }
        }
    }
}

impl Render for Chat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let w = *self.width.lock();
        let h = *self.height.lock();
        let pixels = self.pixels.lock();
        
        let image_element = if w > 0 && h > 0 && !pixels.is_empty() {
            if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, pixels.clone()) {
                let frame = Frame::new(buffer);
                let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));
                
                gpui::img(Arc::new(render_image))
                    .w_full()
                    .h_full()
                    .into_any_element()
            } else {
                gpui::div().into_any_element()
            }
        } else {
            gpui::div()
                .flex()
                .size_full()
                .bg(rgb(0x2e3440))
                .text_color(rgb(0xd8dee9))
                .items_center()
                .justify_center()
                .child(format!("Loading Web Content ({}x{})...", w, h))
                .into_any_element()
        };

        let handle = cx.entity().clone();

        gpui::div()
            .size_full()
            .relative()
            .child(
                gpui::canvas(
                    move |bounds, _, _| bounds,
                    move |_, captured_bounds, _, cx| {
                        let _ = handle.update(cx, |this, _| {
                            this.sync_size(captured_bounds);
                        });
                    }
                )
                .size_full()
                .absolute()
            )
            .child(
                gpui::div()
                    .track_focus(&self.focus_handle)
                    .size_full()
                    .child(image_element)
                    .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, _| {
                        if let Some(host) = this.get_host() {
                            let x = event.position.x.to_f64() as f32;
                            let y = event.position.y.to_f64() as f32;
                            let cef_event = cef::MouseEvent {
                                x: x as i32,
                                y: y as i32,
                                modifiers: 0, 
                            };
                            host.send_mouse_move_event(Some(&cef_event), 0);
                        }
                    }))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &MouseDownEvent, _, _| {
                        if let Some(host) = this.get_host() {
                            let x = event.position.x.to_f64() as f32;
                            let y = event.position.y.to_f64() as f32;
                            let cef_event = cef::MouseEvent {
                                x: x as i32,
                                y: y as i32,
                                modifiers: 0,
                            };
                            host.send_mouse_click_event(Some(&cef_event), MBT_LEFT.into(), 0, 1);
                            host.set_focus(1);
                        }
                    }))
                    .on_mouse_up(MouseButton::Left, cx.listener(move |this, event: &MouseUpEvent, _, _| {
                        if let Some(host) = this.get_host() {
                            let x = event.position.x.to_f64() as f32;
                            let y = event.position.y.to_f64() as f32;
                            let cef_event = cef::MouseEvent {
                                x: x as i32,
                                y: y as i32,
                                modifiers: 0,
                            };
                            host.send_mouse_click_event(Some(&cef_event), MBT_LEFT.into(), 1, 1);
                        }
                    }))
                    .on_scroll_wheel(cx.listener(move |this, event: &ScrollWheelEvent, _, _| {
                        if let Some(host) = this.get_host() {
                            let x = event.position.x.to_f64() as f32;
                            let y = event.position.y.to_f64() as f32;
                            let cef_event = cef::MouseEvent {
                                x: x as i32,
                                y: y as i32,
                                modifiers: 0,
                            };
                            let delta = event.delta.pixel_delta(px(20.0));
                            host.send_mouse_wheel_event(Some(&cef_event), delta.x.to_f64() as i32, delta.y.to_f64() as i32);
                        }
                    }))
            )
    }
}