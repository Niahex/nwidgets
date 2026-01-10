use anyhow::Result;
use cef::{
    args::Args, App, Browser, BrowserSettings, Client, ImplApp, ImplClient,
    ImplRenderHandler, RenderHandler, Settings, WindowInfo, WrapApp, WrapClient, WrapRenderHandler,
    ImplCommandLine, CefString, api_hash, rc::Rc,
};
use gpui::{App as GpuiApp, AsyncApp};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub struct CefService;

impl gpui::Global for CefService {}

impl CefService {
    pub fn init(cx: &mut GpuiApp) {
        cx.set_global(CefService);

        // Schedule CEF message loop work
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;
                    
                    let _ = cx.update(|_| {
                        cef::do_message_loop_work();
                    });
                }
            }
        })
        .detach();
    }
}

#[derive(Clone)]
struct CefAppStruct;

impl CefAppStruct {
    fn new() -> Self {
        Self
    }
}

use cef::wrap_app;

wrap_app! {
    struct AppWrapper {
        app: CefAppStruct,
    }

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut cef::CommandLine>,
        ) {
            if let Some(command_line) = command_line {
                command_line.append_switch(Some(&"disable-gpu".into())); 
                command_line.append_switch(Some(&"disable-gpu-compositing".into()));
                command_line.append_switch(Some(&"enable-begin-frame-scheduling".into()));
                command_line.append_switch(Some(&"no-sandbox".into())); 
            }
        }
    }
}

pub fn initialize_cef() -> Result<()> {
    let _ = api_hash(cef_dll_sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();
    let settings = Settings {
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        ..Default::default()
    };

    let mut app = AppWrapper::new(CefAppStruct::new());

    let code = cef::execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    if code >= 0 {
        std::process::exit(code);
    }

    let result = cef::initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut(),
    );

    if result != 1 {
        return Err(anyhow::anyhow!("Failed to initialize CEF"));
    }

    Ok(())
}

pub fn shutdown_cef() {
    cef::shutdown();
}

#[derive(Clone)]
pub struct GpuiRenderHandler {
    pub pixels: Arc<Mutex<Vec<u8>>>,
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
    pub repaint_callback: Arc<dyn Fn() + Send + Sync>,
}

use cef::wrap_render_handler;

wrap_render_handler! {
    struct GpuiRenderHandlerWrapper {
        handler: GpuiRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut cef::Rect>) {
            if let Some(rect) = rect {
                let w = *self.handler.width.lock();
                let h = *self.handler.height.lock();
                rect.width = w as i32;
                rect.height = h as i32;
            }
        }

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: cef::PaintElementType,
            _dirty_rects: Option<&[cef::Rect]>,
            buffer: *const u8,
            width: i32,
            height: i32,
        ) {
            eprintln!("CEF on_paint called! Size: {}x{}, buffer null: {}", width, height, buffer.is_null());
            if buffer.is_null() {
                return;
            }
            let len = (width * height * 4) as usize;
            let src = unsafe { std::slice::from_raw_parts(buffer, len) };
            let mut pixels = self.handler.pixels.lock();
            if pixels.len() != len {
                eprintln!("Resizing pixel buffer from {} to {}", pixels.len(), len);
                pixels.resize(len, 0);
            }
            pixels.copy_from_slice(src);
            eprintln!("Copied {} bytes to pixel buffer", len);
            (self.handler.repaint_callback)();
        }
    }
}

#[derive(Clone)]
pub struct BrowserClient {
    render_handler: RenderHandler,
}

use cef::wrap_client;

wrap_client! {
    struct ClientWrapper {
        client: BrowserClient,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.client.render_handler.clone())
        }
    }
}

pub fn create_browser(
    url: String,
    pixels: Arc<Mutex<Vec<u8>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
    repaint_callback: Arc<dyn Fn() + Send + Sync>,
) -> Browser {
    let render_handler = GpuiRenderHandler {
        pixels,
        width,
        height,
        repaint_callback,
    };

    let mut client_wrapper = ClientWrapper::new(BrowserClient {
        render_handler: GpuiRenderHandlerWrapper::new(render_handler),
    });

    let window_info = WindowInfo {
        windowless_rendering_enabled: true as _,
        ..Default::default()
    };

    let browser_settings = BrowserSettings {
        windowless_frame_rate: 60,
        ..Default::default()
    };

    let browser = cef::browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client_wrapper),
        Some(&CefString::from(url.as_str())),
        Some(&browser_settings),
        None,
        None,
    );

    browser.expect("Failed to create browser")
}
