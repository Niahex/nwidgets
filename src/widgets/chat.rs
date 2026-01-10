use crate::services::cef::{create_browser, CefService};
use cef::Browser;
use gpui::{App, Context, Entity, IntoElement, Render, Styled, VisualContext, Window, rgb, AppContext, ParentElement, RenderImage, AsyncApp, WeakEntity};
use parking_lot::Mutex;
use std::sync::Arc;
use image::{Frame, ImageBuffer, Rgba};
use smallvec::SmallVec;

pub struct Chat {
    browser: Option<Browser>,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
}

impl Chat {
    // cx is &mut App
    pub fn new(cx: &mut App) -> Entity<Self> {
        // Ensure CEF service is running
        if !cx.has_global::<CefService>() {
            eprintln!("CefService not initialized!");
        }

        cx.new(|cx| {
            let pixels = Arc::new(Mutex::new(Vec::new()));
            let width = Arc::new(Mutex::new(800));
            let height = Arc::new(Mutex::new(600));

            let pixels_clone = pixels.clone();
            let width_clone = width.clone();
            let height_clone = height.clone();

            // Callback to trigger repaint
            let repaint_callback = Arc::new(move || {
                // Do nothing, the timer will pick it up
            });
            
            let browser = create_browser(
                "https://google.com".to_string(),
                pixels_clone,
                width_clone,
                height_clone,
                repaint_callback
            );

            // Refresh loop at 60 FPS
            cx.spawn(|view: WeakEntity<Chat>, cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                let view = view.clone();
                async move {
                     loop {
                         cx.background_executor().timer(std::time::Duration::from_millis(16)).await;
                         // Update the view on the main thread
                         let _ = view.update(&mut cx, |_, cx| cx.notify());
                     }
                }
            }).detach();

            Chat {
                browser: Some(browser),
                pixels,
                width,
                height,
            }
        })
    }
}

impl Render for Chat {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let w = *self.width.lock();
        let h = *self.height.lock();
        let pixels = self.pixels.lock();
        
        if w > 0 && h > 0 && !pixels.is_empty() {
            // Create image buffer. We clone pixels here which is slow but safe for MVP.
            // CEF produces BGRA. GPUI expects BGRA in RenderImage (stored in Frame).
            // We pretend it's RgbaImage because Frame expects it, but the data is BGRA.
            if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, pixels.clone()) {
                let frame = Frame::new(buffer);
                let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));
                
                return gpui::img(Arc::new(render_image))
                    .w_full()
                    .h_full()
                    .into_any_element();
            }
        }
        
        gpui::div()
            .flex()
            .size_full()
            .bg(rgb(0x2e3440))
            .text_color(rgb(0xd8dee9))
            .items_center()
            .justify_center()
            .child(format!("CEF Chat Widget ({}x{}) - No Image", w, h))
            .into_any_element()
    }
}
