use crate::services::cef::handlers::{
    CefCursor, DisplayHandlerWrapper, GpuiDisplayHandler, GpuiPermissionHandler,
    GpuiRenderHandler, PermissionHandlerWrapper, RenderHandlerWrapper,
};
use crate::services::cef::input::{key_to_windows_code, modifiers_to_cef, send_char_event, send_key_event, SCROLL_MULTIPLIER};
use cef::{
    rc::Rc, Browser, BrowserSettings, CefString, Client, DisplayHandler, ImplBrowser, ImplBrowserHost,
    ImplClient, ImplFrame, PermissionHandler, RenderHandler, WindowInfo, WrapClient,
};
use cef_dll_sys::cef_mouse_button_type_t;
use gpui::{
    div, img, px, rgb, AsyncApp, Context, CursorStyle, ExternalPaths, Focusable, FocusHandle,
    InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, RenderImage, ScrollWheelEvent, Styled, WeakEntity,
    Window,
};
use image::{Frame, ImageBuffer, Rgba};
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
struct BrowserClient {
    render_handler: RenderHandler,
    display_handler: DisplayHandler,
    permission_handler: PermissionHandler,
}

cef::wrap_client! {
    struct ClientWrapper {
        client: BrowserClient,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.client.render_handler.clone())
        }
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(self.client.display_handler.clone())
        }
        fn permission_handler(&self) -> Option<PermissionHandler> {
            Some(self.client.permission_handler.clone())
        }
    }
}

pub fn create_browser(
    url: String,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    cursor: Arc<Mutex<CefCursor>>,
) -> Browser {
    let render_handler = RenderHandlerWrapper::new(GpuiRenderHandler { pixels, width, height });
    let display_handler = DisplayHandlerWrapper::new(GpuiDisplayHandler { cursor });
    let permission_handler = PermissionHandlerWrapper::new(GpuiPermissionHandler);

    let mut client = ClientWrapper::new(BrowserClient {
        render_handler,
        display_handler,
        permission_handler,
    });

    let window_info = WindowInfo {
        windowless_rendering_enabled: true as _,
        ..Default::default()
    };

    let browser_settings = BrowserSettings {
        windowless_frame_rate: 60,
        javascript_access_clipboard: cef::State::ENABLED,
        ..Default::default()
    };

    cef::browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client),
        Some(&CefString::from(url.as_str())),
        Some(&browser_settings),
        None,
        None,
    )
    .expect("Failed to create browser")
}

pub struct BrowserView {
    browser: Option<Browser>,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    focus_handle: FocusHandle,
    mouse_pressed: Arc<Mutex<bool>>,
    cursor: Arc<Mutex<CefCursor>>,
    cached_image: Arc<Mutex<Option<Arc<RenderImage>>>>,
    last_hash: Arc<Mutex<u64>>,
}

impl BrowserView {
    pub fn new(url: &str, width: u32, height: u32, cx: &mut Context<Self>) -> Self {
        let pixels = Arc::new(Mutex::new(Vec::new()));
        let w = Arc::new(Mutex::new(width));
        let h = Arc::new(Mutex::new(height));
        let mouse_pressed = Arc::new(Mutex::new(false));
        let cursor = Arc::new(Mutex::new(CefCursor::Default));

        let browser = create_browser(url.to_string(), pixels.clone(), w.clone(), h.clone(), cursor.clone());

        if let Some(host) = browser.host() {
            host.was_resized();
            host.set_focus(1);
        }

        cx.spawn(|view: WeakEntity<BrowserView>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(16)).await;
                    let _ = view.update(&mut cx, |_, cx| cx.notify());
                }
            }
        })
        .detach();

        Self {
            browser: Some(browser),
            pixels,
            width: w,
            height: h,
            focus_handle: cx.focus_handle(),
            mouse_pressed,
            cursor,
            cached_image: Arc::new(Mutex::new(None)),
            last_hash: Arc::new(Mutex::new(0)),
        }
    }

    fn send_key(&self, key_code: i32, modifiers: u32, down: bool) {
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                send_key_event(&host, key_code, modifiers, down);
            }
        }
    }

    fn send_char(&self, ch: char, modifiers: u32) {
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                send_char_event(&host, ch, modifiers);
            }
        }
    }
}

impl Focusable for BrowserView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for BrowserView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let w = *self.width.lock();
        let h = *self.height.lock();
        let pixels = self.pixels.lock();
        let browser = self.browser.clone();
        let mouse_pressed = self.mouse_pressed.clone();

        let cursor_style = match *self.cursor.lock() {
            CefCursor::Default => CursorStyle::Arrow,
            CefCursor::Pointer => CursorStyle::PointingHand,
            CefCursor::Text => CursorStyle::IBeam,
            CefCursor::Move => CursorStyle::ClosedHand,
            CefCursor::Wait | CefCursor::None => CursorStyle::Arrow,
        };

        if w > 0 && h > 0 && !pixels.is_empty() {
            let mut hasher = DefaultHasher::new();
            pixels.hash(&mut hasher);
            let current_hash = hasher.finish();

            let render_image = {
                let mut last = self.last_hash.lock();
                let mut cached = self.cached_image.lock();

                if *last != current_hash || cached.is_none() {
                    if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, pixels.clone()) {
                        let frame = Frame::new(buffer);
                        let img = Arc::new(RenderImage::new(SmallVec::from_elem(frame, 1)));
                        *cached = Some(img.clone());
                        *last = current_hash;
                        img
                    } else {
                        return loading_view(w, h);
                    }
                } else {
                    cached.as_ref().unwrap().clone()
                }
            };

            let browser_move = browser.clone();
            let browser_down = browser.clone();
            let browser_up = browser.clone();
            let browser_scroll = browser.clone();
            let mouse_pressed_move = mouse_pressed.clone();
            let mouse_pressed_down = mouse_pressed.clone();
            let mouse_pressed_up = mouse_pressed.clone();

            return div()
                .size_full()
                .cursor(cursor_style)
                .track_focus(&self.focus_handle)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, _cx| {
                    let keystroke = &event.keystroke;
                    let modifiers = modifiers_to_cef(&keystroke.modifiers);

                    if keystroke.modifiers.control && keystroke.key == "v" {
                        if let Some(browser) = &this.browser {
                            if let Some(frame) = browser.main_frame() { frame.paste(); }
                        }
                        return;
                    }

                    if keystroke.modifiers.control && keystroke.key == "f" {
                        this.send_key(70, modifiers, true);
                        this.send_key(70, modifiers, false);
                        return;
                    }

                    if keystroke.modifiers.control && matches!(keystroke.key.as_str(), "c" | "a" | "x") {
                        if let Some(browser) = &this.browser {
                            if let Some(frame) = browser.main_frame() {
                                match keystroke.key.as_str() {
                                    "c" => frame.copy(),
                                    "x" => frame.cut(),
                                    "a" => frame.select_all(),
                                    _ => {}
                                }
                            }
                        }
                        return;
                    }

                    if let Some(code) = key_to_windows_code(&keystroke.key) {
                        this.send_key(code, modifiers, true);
                    }

                    if !keystroke.modifiers.control && !keystroke.modifiers.alt {
                        if let Some(ch) = keystroke.key_char.as_ref().and_then(|s| s.chars().next()) {
                            this.send_char(ch, modifiers);
                        } else if keystroke.key.len() == 1 {
                            let ch = keystroke.key.chars().next().unwrap();
                            let ch = if keystroke.modifiers.shift && ch.is_ascii_alphabetic() {
                                ch.to_ascii_uppercase()
                            } else { ch };
                            this.send_char(ch, modifiers);
                        }
                    }
                }))
                .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, _cx| {
                    let keystroke = &event.keystroke;
                    if let Some(code) = key_to_windows_code(&keystroke.key) {
                        this.send_key(code, modifiers_to_cef(&keystroke.modifiers), false);
                    }
                }))
                .on_mouse_move(move |event: &MouseMoveEvent, _window, _cx| {
                    if let Some(browser) = &browser_move {
                        if let Some(host) = browser.host() {
                            let pos = event.position;
                            host.send_mouse_move_event(Some(&cef::MouseEvent {
                                x: Into::<f32>::into(pos.x) as i32,
                                y: Into::<f32>::into(pos.y) as i32,
                                modifiers: if *mouse_pressed_move.lock() { 16 } else { 0 },
                            }), 0);
                        }
                    }
                })
                .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _window, _cx| {
                    *mouse_pressed_down.lock() = true;
                    if let Some(browser) = &browser_down {
                        if let Some(host) = browser.host() {
                            let pos = event.position;
                            host.send_mouse_click_event(Some(&cef::MouseEvent {
                                x: Into::<f32>::into(pos.x) as i32,
                                y: Into::<f32>::into(pos.y) as i32,
                                modifiers: 16,
                            }), cef_mouse_button_type_t::MBT_LEFT.into(), 0, event.click_count as i32);
                        }
                    }
                })
                .on_mouse_up(MouseButton::Left, move |event: &MouseUpEvent, _window, _cx| {
                    *mouse_pressed_up.lock() = false;
                    if let Some(browser) = &browser_up {
                        if let Some(host) = browser.host() {
                            let pos = event.position;
                            host.send_mouse_click_event(Some(&cef::MouseEvent {
                                x: Into::<f32>::into(pos.x) as i32,
                                y: Into::<f32>::into(pos.y) as i32,
                                modifiers: 0,
                            }), cef_mouse_button_type_t::MBT_LEFT.into(), 1, event.click_count as i32);
                        }
                    }
                })
                .on_mouse_down(MouseButton::Right, {
                    let browser = browser.clone();
                    move |event: &MouseDownEvent, _window, _cx| {
                        if let Some(browser) = &browser {
                            if let Some(host) = browser.host() {
                                let pos = event.position;
                                host.send_mouse_click_event(Some(&cef::MouseEvent {
                                    x: Into::<f32>::into(pos.x) as i32,
                                    y: Into::<f32>::into(pos.y) as i32,
                                    modifiers: 32,
                                }), cef_mouse_button_type_t::MBT_RIGHT.into(), 0, 1);
                            }
                        }
                    }
                })
                .on_mouse_up(MouseButton::Right, {
                    let browser = browser.clone();
                    move |event: &MouseUpEvent, _window, _cx| {
                        if let Some(browser) = &browser {
                            if let Some(host) = browser.host() {
                                let pos = event.position;
                                host.send_mouse_click_event(Some(&cef::MouseEvent {
                                    x: Into::<f32>::into(pos.x) as i32,
                                    y: Into::<f32>::into(pos.y) as i32,
                                    modifiers: 0,
                                }), cef_mouse_button_type_t::MBT_RIGHT.into(), 1, 1);
                            }
                        }
                    }
                })
                .on_scroll_wheel(move |event: &ScrollWheelEvent, _window, _cx| {
                    if let Some(browser) = &browser_scroll {
                        if let Some(host) = browser.host() {
                            let pos = event.position;
                            let delta = event.delta.pixel_delta(px(1.0));
                            host.send_mouse_wheel_event(Some(&cef::MouseEvent {
                                x: Into::<f32>::into(pos.x) as i32,
                                y: Into::<f32>::into(pos.y) as i32,
                                modifiers: 0,
                            }), (Into::<f32>::into(delta.x) as i32) * SCROLL_MULTIPLIER,
                               (Into::<f32>::into(delta.y) as i32) * SCROLL_MULTIPLIER);
                        }
                    }
                })
                .on_mouse_down(MouseButton::Navigate(gpui::NavigationDirection::Back), {
                    let browser = browser.clone();
                    move |_, _, _| { if let Some(b) = &browser { b.go_back(); } }
                })
                .on_mouse_down(MouseButton::Navigate(gpui::NavigationDirection::Forward), {
                    let browser = browser.clone();
                    move |_, _, _| { if let Some(b) = &browser { b.go_forward(); } }
                })
                .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, _cx| {
                    if let Some(browser) = &this.browser {
                        if let Some(frame) = browser.main_frame() {
                            if let Some(path) = paths.paths().first() {
                                let script = format!(
                                    "window.dispatchEvent(new CustomEvent('filedrop', {{detail: '{}'}}));",
                                    path.to_string_lossy().replace('\'', "\\'")
                                );
                                frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0);
                            }
                        }
                    }
                }))
                .child(img(render_image).w_full().h_full())
                .into_any_element();
        }

        loading_view(w, h)
    }
}

fn loading_view(w: u32, h: u32) -> gpui::AnyElement {
    div()
        .flex()
        .size_full()
        .bg(rgb(0x2e3440))
        .text_color(rgb(0xd8dee9))
        .items_center()
        .justify_center()
        .child(format!("Loading... ({}x{})", w, h))
        .into_any_element()
}
