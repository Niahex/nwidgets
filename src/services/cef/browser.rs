use crate::services::cef::find::FindBar;
use crate::services::cef::handlers::{
    CefCursor, DisplayHandlerWrapper, DoubleBuffer, GpuiDisplayHandler, GpuiLoadHandler,
    GpuiPermissionHandler, GpuiRenderHandler, LoadHandlerWrapper, PermissionHandlerWrapper,
    RenderHandlerWrapper,
};
use crate::services::cef::input::{
    key_to_windows_code, modifiers_to_cef, send_char_event, send_key_event, SCROLL_MULTIPLIER,
};
use cef::{
    rc::Rc, Browser, BrowserSettings, CefString, Client, DisplayHandler, ImplBrowser,
    ImplBrowserHost, ImplClient, ImplFrame, LoadHandler, PermissionHandler, ProcessId,
    ProcessMessage, RenderHandler, WindowInfo, WrapClient,
};
use cef::wrapper::message_router::{
    BrowserSideRouter, MessageRouterBrowserSide, MessageRouterBrowserSideHandlerCallbacks,
    MessageRouterConfig,
};
use crate::services::cef::message_handler::ClipboardMessageHandler;
use cef_dll_sys::cef_mouse_button_type_t;
use futures::StreamExt;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, rgb, AsyncApp, Context, CursorStyle, ExternalPaths, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, RenderImage, ScrollWheelEvent, Styled, WeakEntity,
    Window,
};
use image::{Frame, ImageBuffer, Rgba};
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::sync::Arc;

#[derive(Clone)]
struct BrowserClient {
    render_handler: RenderHandler,
    display_handler: DisplayHandler,
    permission_handler: PermissionHandler,
    load_handler: LoadHandler,
    message_router: std::sync::Arc<BrowserSideRouter>,
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
        fn load_handler(&self) -> Option<LoadHandler> {
            Some(self.client.load_handler.clone())
        }
        fn on_process_message_received(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut cef::Frame>,
            source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> i32 {
            // Convert mutable references to owned types for the router
            let browser_owned = browser.as_ref().map(|b| (*b).clone());
            let frame_owned = frame.as_ref().map(|f| (*f).clone());
            let message_owned = message.as_ref().map(|m| (*m).clone());
            
            if self.client.message_router.on_process_message_received(
                browser_owned,
                frame_owned,
                source_process,
                message_owned,
            ) {
                1 // true
            } else {
                0 // false
            }
        }
    }
}

struct BrowserConfig {
    buffer: Arc<DoubleBuffer>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    cursor: Arc<Mutex<CefCursor>>,
    selected_text: Arc<Mutex<String>>,
    injection_script: Arc<Mutex<Option<String>>>,
    loaded: Arc<Mutex<bool>>,
    scale_factor: f32,
    repaint_tx: futures::channel::mpsc::UnboundedSender<()>,
    clipboard_tx: futures::channel::mpsc::UnboundedSender<super::clipboard::ClipboardData>,
}

fn create_browser(url: &str, config: BrowserConfig) -> (Browser, std::sync::Arc<BrowserSideRouter>) {
    // Create MessageRouter for clipboard communication
    let router_config = MessageRouterConfig::default();
    let message_router = BrowserSideRouter::new(router_config);
    
    // Create and add clipboard handler
    let clipboard_handler = std::sync::Arc::new(ClipboardMessageHandler::new(config.clipboard_tx.clone()));
    message_router.add_handler(clipboard_handler, true);
    
    let render_handler = RenderHandlerWrapper::new(GpuiRenderHandler {
        buffer: config.buffer,
        width: config.width,
        height: config.height,
        scale_factor: config.scale_factor,
        selected_text: config.selected_text,
        repaint_tx: config.repaint_tx,
    });
    let display_handler = DisplayHandlerWrapper::new(GpuiDisplayHandler {
        cursor: config.cursor,
        clipboard_tx: config.clipboard_tx,
    });
    let permission_handler = PermissionHandlerWrapper::new(GpuiPermissionHandler);
    let load_handler = LoadHandlerWrapper::new(GpuiLoadHandler {
        injection_script: config.injection_script,
        loaded: config.loaded,
    });

    let mut client = ClientWrapper::new(BrowserClient {
        render_handler,
        display_handler,
        permission_handler,
        load_handler,
        message_router: message_router.clone(),
    });

    let browser = cef::browser_host_create_browser_sync(
        Some(&WindowInfo {
            windowless_rendering_enabled: 1,
            ..Default::default()
        }),
        Some(&mut client),
        Some(&CefString::from(url)),
        Some(&BrowserSettings {
            windowless_frame_rate: 60,
            javascript_access_clipboard: cef::State::ENABLED,
            javascript_dom_paste: cef::State::ENABLED,
            ..Default::default()
        }),
        None,
        None,
    )
    .expect("Failed to create browser");
    
    (browser, message_router)
}

pub struct BrowserView {
    browser: Option<Browser>,
    buffer: Arc<DoubleBuffer>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    focus_handle: FocusHandle,
    mouse_pressed: Arc<Mutex<bool>>,
    cursor: Arc<Mutex<CefCursor>>,
    selected_text: Arc<Mutex<String>>,
    loaded: Arc<Mutex<bool>>,
    hidden: Arc<Mutex<bool>>,
    last_version: u64,
    cached_image: Option<Arc<RenderImage>>,
    reuse_buffer: Vec<u8>,
    find_bar: FindBar,
    message_router: std::sync::Arc<BrowserSideRouter>,
}

impl BrowserView {
    pub fn new(
        url: &str,
        width: u32,
        height: u32,
        injection_script: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Self {
        let buffer = Arc::new(DoubleBuffer::new((width * height * 4) as usize));
        let w = Arc::new(Mutex::new(width));
        let h = Arc::new(Mutex::new(height));
        let mouse_pressed = Arc::new(Mutex::new(false));
        let cursor = Arc::new(Mutex::new(CefCursor::Default));
        let selected_text = Arc::new(Mutex::new(String::new()));

        // Combine clipboard script with user injection script
        let full_script = match injection_script {
            Some(script) => format!("{}{}", super::clipboard::CLIPBOARD_SCRIPT, script),
            None => super::clipboard::CLIPBOARD_SCRIPT.to_string(),
        };
        let script_arc = Arc::new(Mutex::new(Some(full_script)));

        let loaded = Arc::new(Mutex::new(false));
        let hidden = Arc::new(Mutex::new(false));

        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        let (clipboard_tx, clipboard_rx) = super::clipboard::create_clipboard_channel();

        let (browser, message_router) = create_browser(
            url,
            BrowserConfig {
                buffer: buffer.clone(),
                width: w.clone(),
                height: h.clone(),
                cursor: cursor.clone(),
                selected_text: selected_text.clone(),
                injection_script: script_arc,
                loaded: loaded.clone(),
                scale_factor: 1.0,
                repaint_tx: tx,
                clipboard_tx,
            },
        );

        if let Some(host) = browser.host() {
            host.was_resized();
            host.set_focus(1);
        }

        // Clipboard handler
        super::clipboard::spawn_clipboard_handler(cx, clipboard_rx);

        // Event-driven repaint loop (Push)
        cx.spawn(move |view: WeakEntity<BrowserView>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while rx.next().await.is_some() {
                    let _ = view.update(&mut cx, |_, cx| cx.notify());
                }
            }
        })
        .detach();

        Self {
            browser: Some(browser),
            buffer,
            width: w,
            height: h,
            focus_handle: cx.focus_handle(),
            mouse_pressed,
            cursor,
            selected_text,
            loaded,
            hidden,
            last_version: 0,
            cached_image: None,
            reuse_buffer: Vec::new(),
            find_bar: FindBar::new(),
            message_router,
        }
    }

    #[inline]
    fn send_key(&self, key_code: i32, modifiers: u32, down: bool) {
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                send_key_event(&host, key_code, modifiers, down);
            }
        }
    }

    #[inline]
    fn send_char(&self, ch: char, modifiers: u32) {
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                send_char_event(&host, ch, modifiers);
            }
        }
    }

    pub fn current_url(&self) -> Option<String> {
        self.browser.as_ref().and_then(|b| b.main_frame()).map(|f| {
            let cef_str: cef::CefStringUtf16 = (&f.url()).into();
            format!("{cef_str}")
        })
    }

    pub fn navigate(&self, url: &str) {
        if let Some(browser) = &self.browser {
            if let Some(frame) = browser.main_frame() {
                frame.load_url(Some(&CefString::from(url)));
            }
        }
    }

    pub fn reload(&self) {
        if let Some(browser) = &self.browser {
            browser.reload();
        }
    }

    pub fn execute_js(&self, js: &str) {
        if let Some(browser) = &self.browser {
            if let Some(frame) = browser.main_frame() {
                frame.execute_java_script(Some(&CefString::from(js)), None, 0);
            }
        }
    }

    pub fn set_hidden(&self, hidden: bool) {
        *self.hidden.lock() = hidden;
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                host.was_hidden(hidden as i32);
            }
        }
    }

    pub fn resize(&self, width: u32, height: u32) {
        *self.width.lock() = width;
        *self.height.lock() = height;
        if let Some(browser) = &self.browser {
            if let Some(host) = browser.host() {
                host.was_resized();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_hidden = *self.hidden.lock();

        if is_hidden {
            return div().into_any_element();
        }

        let w = *self.width.lock();
        let h = *self.height.lock();
        let current_version = self.buffer.version();
        let is_loaded = *self.loaded.lock();

        let cursor_style = match *self.cursor.lock() {
            CefCursor::Default => CursorStyle::Arrow,
            CefCursor::Pointer => CursorStyle::PointingHand,
            CefCursor::Text => CursorStyle::IBeam,
            CefCursor::Move => CursorStyle::ClosedHand,
            CefCursor::Wait | CefCursor::None => CursorStyle::Arrow,
        };

        if w > 0 && h > 0 {
            // Only rebuild image if version changed
            if current_version != self.last_version || self.cached_image.is_none() {
                let pixels = self.buffer.read();
                let expected_len = (w * h * 4) as usize;
                if pixels.len() == expected_len {
                    // Reuse buffer allocation
                    self.reuse_buffer.clear();
                    self.reuse_buffer.extend_from_slice(&pixels);
                    drop(pixels);

                    if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                        w,
                        h,
                        std::mem::take(&mut self.reuse_buffer),
                    ) {
                        if let Some(old_image) = self.cached_image.take() {
                            cx.drop_image(old_image, Some(window));
                        }
                        self.cached_image = Some(Arc::new(RenderImage::new(SmallVec::from_elem(
                            Frame::new(buffer),
                            1,
                        ))));
                        self.last_version = current_version;
                    }
                }
            }

            if let Some(ref render_image) = self.cached_image {
                let browser = self.browser.clone();
                let mouse_pressed = self.mouse_pressed.clone();

                let mut main_div = div()
                    .size_full()
                    .cursor(cursor_style)
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        let ks = &event.keystroke;
                        let mods = modifiers_to_cef(&ks.modifiers);

                        // Handle Ctrl+V / Cmd+V for paste
                        if (ks.modifiers.control || ks.modifiers.platform) && ks.key == "v" {
                            if let Some(browser) = &this.browser {
                                if let Some(clipboard_item) = cx.read_from_clipboard() {
                                    super::clipboard_inject::inject_clipboard_to_cef(browser, &clipboard_item);
                                }
                                return;
                            }
                        }

                        // Handle find bar input
                        if this.find_bar.visible {
                            if let Some(b) = &this.browser {
                                if let Some(host) = b.host() {
                                    if this.find_bar.handle_key(&ks.key, ks.key_char.as_deref(), &ks.modifiers, &host) {
                                        cx.notify();
                                        return;
                                    }
                                }
                            }
                        }

                        // F5 for reload
                        if ks.key == "f5" {
                            this.reload();
                            return;
                        }

                        // Ctrl+Shift+I for DevTools
                        if ks.modifiers.control && ks.modifiers.shift && ks.key == "i" {
                            if let Some(b) = &this.browser {
                                if let Some(host) = b.host() {
                                    host.show_dev_tools(None, None, None, None);
                                }
                            }
                            return;
                        }

                        if ks.modifiers.control {
                            match ks.key.as_str() {
                                "v" => {
                                    if ks.modifiers.shift {
                                        // Ctrl+Shift+V: Paste without formatting
                                        if let Some(b) = &this.browser {
                                            if let Some(f) = b.main_frame() {
                                                f.paste_and_match_style();
                                            }
                                        }
                                    }
                                    return;
                                }
                                "z" => {
                                    if ks.modifiers.shift {
                                        // Ctrl+Shift+Z: Redo
                                        if let Some(b) = &this.browser {
                                            if let Some(f) = b.main_frame() {
                                                f.redo();
                                            }
                                        }
                                    } else {
                                        // Ctrl+Z: Undo
                                        if let Some(b) = &this.browser {
                                            if let Some(f) = b.main_frame() {
                                                f.undo();
                                            }
                                        }
                                    }
                                    return;
                                }
                                "y" => {
                                    // Ctrl+Y: Redo (alternative)
                                    if let Some(b) = &this.browser {
                                        if let Some(f) = b.main_frame() {
                                            f.redo();
                                        }
                                    }
                                    return;
                                }
                                "c" => {
                                    if let Some(b) = &this.browser {
                                        if let Some(f) = b.main_frame() {
                                            f.copy();
                                            // Send selected text to system clipboard
                                            let text = this.selected_text.lock().clone();
                                            if !text.is_empty() {
                                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                                            }
                                        }
                                    }
                                    return;
                                }
                                "x" => {
                                    if let Some(b) = &this.browser {
                                        if let Some(f) = b.main_frame() {
                                            // Send selected text to system clipboard before cutting
                                            let text = this.selected_text.lock().clone();
                                            if !text.is_empty() {
                                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                                            }
                                            f.cut();
                                        }
                                    }
                                    return;
                                }
                                "a" => { if let Some(b) = &this.browser { if let Some(f) = b.main_frame() { f.select_all(); } } return; }
                                "f" => { 
                                    this.find_bar.toggle();
                                    if !this.find_bar.visible {
                                        if let Some(b) = &this.browser {
                                            if let Some(host) = b.host() {
                                                host.stop_finding(1);
                                            }
                                        }
                                    }
                                    cx.notify();
                                    return;
                                }
                                _ => {}
                            }
                        }

                        if let Some(code) = key_to_windows_code(&ks.key) {
                            this.send_key(code, mods, true);
                        }

                        if !ks.modifiers.control && !ks.modifiers.alt {
                            if let Some(ch) = ks.key_char.as_ref().and_then(|s| s.chars().next()) {
                                this.send_char(ch, mods);
                            } else if ks.key.len() == 1 {
                                let ch = ks.key.chars().next().unwrap();
                                this.send_char(if ks.modifiers.shift && ch.is_ascii_alphabetic() { ch.to_ascii_uppercase() } else { ch }, mods);
                            }
                        }
                    }))
                    .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, _cx| {
                        if let Some(code) = key_to_windows_code(&event.keystroke.key) {
                            this.send_key(code, modifiers_to_cef(&event.keystroke.modifiers), false);
                        }
                    }))
                    .on_mouse_move({
                        let browser = browser.clone();
                        let mouse_pressed = mouse_pressed.clone();
                        move |event: &MouseMoveEvent, _window, _cx| {
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    host.send_mouse_move_event(Some(&cef::MouseEvent {
                                        x, y, modifiers: if *mouse_pressed.lock() { 16 } else { 0 },
                                    }), 0);
                                }
                            }
                        }
                    })
                    .on_mouse_down(MouseButton::Left, {
                        let browser = browser.clone();
                        let mouse_pressed = mouse_pressed.clone();
                        move |event: &MouseDownEvent, _window, _cx| {
                            *mouse_pressed.lock() = true;
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    host.set_focus(1);
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    host.send_mouse_click_event(Some(&cef::MouseEvent { x, y, modifiers: 16 }),
                                        cef_mouse_button_type_t::MBT_LEFT.into(), 0, event.click_count as i32);
                                }
                            }
                        }
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let browser = browser.clone();
                        let mouse_pressed = mouse_pressed.clone();
                        move |event: &MouseUpEvent, _window, _cx| {
                            *mouse_pressed.lock() = false;
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    host.send_mouse_click_event(Some(&cef::MouseEvent { x, y, modifiers: 0 }),
                                        cef_mouse_button_type_t::MBT_LEFT.into(), 1, event.click_count as i32);
                                }
                            }
                        }
                    })
                    .on_mouse_down(MouseButton::Right, {
                        let browser = browser.clone();
                        move |event: &MouseDownEvent, _window, _cx| {
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    host.send_mouse_click_event(Some(&cef::MouseEvent { x, y, modifiers: 32 }),
                                        cef_mouse_button_type_t::MBT_RIGHT.into(), 0, 1);
                                }
                            }
                        }
                    })
                    .on_mouse_up(MouseButton::Right, {
                        let browser = browser.clone();
                        move |event: &MouseUpEvent, _window, _cx| {
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    host.send_mouse_click_event(Some(&cef::MouseEvent { x, y, modifiers: 0 }),
                                        cef_mouse_button_type_t::MBT_RIGHT.into(), 1, 1);
                                }
                            }
                        }
                    })
                    .on_scroll_wheel({
                        let browser = browser.clone();
                        move |event: &ScrollWheelEvent, _window, _cx| {
                            if let Some(browser) = &browser {
                                if let Some(host) = browser.host() {
                                    let (x, y) = (Into::<f32>::into(event.position.x) as i32, Into::<f32>::into(event.position.y) as i32);
                                    let delta = event.delta.pixel_delta(px(1.0));
                                    host.send_mouse_wheel_event(Some(&cef::MouseEvent { x, y, modifiers: 0 }),
                                        (Into::<f32>::into(delta.x) as i32) * SCROLL_MULTIPLIER,
                                        (Into::<f32>::into(delta.y) as i32) * SCROLL_MULTIPLIER);
                                }
                            }
                        }
                    })
                    .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, _cx| {
                        if let Some(browser) = &this.browser {
                            if let Some(frame) = browser.main_frame() {
                                if let Some(path) = paths.paths().first() {
                                    let script = format!(
                                        "window.dispatchEvent(new CustomEvent('filedrop', {{detail: '{}'}}));",
                                        path.to_string_lossy().replace('`', "\\`")
                                    );
                                    frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0);
                                }
                            }
                        }
                    }))
                    .child(
                        div()
                            .size_full()
                            .relative()
                            .child(img(render_image.clone()).w_full().h_full().rounded(gpui::px(18.)))
                            .when(!is_loaded, |el| {
                                el.child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(rgb(0x2e3440))
                                        .child("Loading...")
                                )
                            })
                    );

                if self.find_bar.visible {
                    let browser = self.browser.clone();
                    let browser2 = self.browser.clone();
                    let browser3 = self.browser.clone();
                    let query = self.find_bar.query.clone();
                    let query2 = self.find_bar.query.clone();

                    main_div = main_div.child(self.find_bar.render(
                        move |_, _, _| {
                            if let Some(b) = &browser {
                                if let Some(host) = b.host() {
                                    host.find(Some(&CefString::from(query.as_str())), 0, 0, 1);
                                }
                            }
                        },
                        move |_, _, _| {
                            if let Some(b) = &browser2 {
                                if let Some(host) = b.host() {
                                    host.find(Some(&CefString::from(query2.as_str())), 1, 0, 1);
                                }
                            }
                        },
                        move |_, window, _| {
                            if let Some(b) = &browser3 {
                                if let Some(host) = b.host() {
                                    host.stop_finding(1);
                                }
                            }
                            window.refresh();
                        },
                    ));
                }

                return main_div.into_any_element();
            }
        }

        div()
            .flex()
            .size_full()
            .bg(rgb(0x2e3440))
            .text_color(rgb(0xd8dee9))
            .items_center()
            .justify_center()
            .child(format!("Loading... ({w}x{h})"))
            .into_any_element()
    }
}
