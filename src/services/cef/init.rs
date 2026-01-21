use anyhow::Result;
use cef::{
    api_hash, args::Args, rc::Rc, App, CefString, ImplApp, ImplCommandLine, RenderProcessHandler,
    Settings, WrapApp,
};
use gpui::{App as GpuiApp, AsyncApp};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::render_handler::{GpuiRenderProcessHandler, RenderProcessHandlerWrapper};

static ACTIVE_BROWSERS: AtomicUsize = AtomicUsize::new(0);

pub fn register_browser() {
    ACTIVE_BROWSERS.fetch_add(1, Ordering::Relaxed);
}

pub fn unregister_browser() {
    ACTIVE_BROWSERS.fetch_sub(1, Ordering::Relaxed);
}

pub struct CefService;

impl gpui::Global for CefService {}

impl CefService {
    pub fn init(cx: &mut GpuiApp) {
        cx.set_global(CefService);

        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    let active = ACTIVE_BROWSERS.load(Ordering::Relaxed);
                    if active > 0 {
                        // 60Hz when browsers are active
                        cx.background_executor()
                            .timer(Duration::from_millis(16))
                            .await;
                        cx.update(|_| {
                            cef::do_message_loop_work();
                        });
                    } else {
                        // 1Hz when idle to keep CEF alive
                        cx.background_executor()
                            .timer(Duration::from_secs(1))
                            .await;
                        cx.update(|_| {
                            cef::do_message_loop_work();
                        });
                    }
                }
            }
        })
        .detach();
    }
}

#[derive(Clone)]
struct CefAppStruct {
    render_process_handler: RenderProcessHandler,
}

cef::wrap_app! {
    struct AppWrapper {
        app: CefAppStruct,
    }

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut cef::CommandLine>,
        ) {
            if let Some(cmd) = command_line {
                // Check for NWIDGETS_GPU environment variable set by flake.nix
                let gpu_vendor = std::env::var("NWIDGETS_GPU").unwrap_or_else(|_| "unknown".to_string());
                println!("[nwidgets] CEF Init - Detected GPU vendor: {}", gpu_vendor);

                cmd.append_switch(Some(&"enable-begin-frame-scheduling".into()));
                cmd.append_switch(Some(&"no-sandbox".into()));
                cmd.append_switch(Some(&"enable-media-stream".into()));
                cmd.append_switch_with_value(
                    Some(&"ozone-platform".into()),
                    Some(&"wayland".into()),
                );

                if gpu_vendor == "nvidia" {
                    // NVIDIA specific optimizations to prevent freezes
                    println!("[nwidgets] Applying NVIDIA-specific CEF flags...");
                    // Ensure GPU is disabled to prevent EGL conflicts which cause freezes on Nvidia
                    cmd.append_switch(Some(&"disable-gpu".into()));
                    cmd.append_switch(Some(&"disable-gpu-compositing".into()));
                    cmd.append_switch_with_value(
                        Some(&"use-gl".into()),
                        Some(&"swiftshader".into()),
                    );
                    // Additional flags that might help stability on Nvidia
                    cmd.append_switch(Some(&"disable-vulkan".into()));
                } else if gpu_vendor == "amd" {
                    // AMD specific (potentially allow more GPU usage if safe, but stick to safe defaults for now)
                    println!("[nwidgets] Applying AMD-specific CEF flags...");
                     // Force software rendering to avoid EGL conflicts with GPUI (same as default for now)
                    cmd.append_switch(Some(&"disable-gpu".into()));
                    cmd.append_switch(Some(&"disable-gpu-compositing".into()));
                    cmd.append_switch_with_value(
                        Some(&"use-gl".into()),
                        Some(&"swiftshader".into()),
                    );
                } else {
                    // Default / Intel / Unknown
                    // Force software rendering to avoid EGL conflicts with GPUI
                    cmd.append_switch(Some(&"disable-gpu".into()));
                    cmd.append_switch(Some(&"disable-gpu-compositing".into()));
                    cmd.append_switch_with_value(
                        Some(&"use-gl".into()),
                        Some(&"swiftshader".into()),
                    );
                }

                // PipeWire for screen capture and audio
                cmd.append_switch(Some(&"enable-features=WebRTCPipeWireCapturer,SmoothScrolling".into()));
                cmd.append_switch_with_value(
                    Some(&"alsa-output-device".into()),
                    Some(&"pipewire".into()),
                );
                cmd.append_switch_with_value(
                    Some(&"alsa-input-device".into()),
                    Some(&"pipewire".into()),
                );
                // Memory/CPU optimizations
                cmd.append_switch(Some(&"disable-extensions".into()));
                cmd.append_switch(Some(&"disable-background-networking".into()));
                cmd.append_switch(Some(&"disable-sync".into()));
                cmd.append_switch(Some(&"disable-translate".into()));
                cmd.append_switch(Some(&"disable-default-apps".into()));
                cmd.append_switch(Some(&"disable-component-update".into()));
                cmd.append_switch(Some(&"disable-domain-reliability".into()));
                cmd.append_switch(Some(&"disable-client-side-phishing-detection".into()));
                cmd.append_switch(Some(&"disable-hang-monitor".into()));
                cmd.append_switch(Some(&"disable-popup-blocking".into()));
                cmd.append_switch(Some(&"disable-prompt-on-repost".into()));
                cmd.append_switch(Some(&"disable-breakpad".into()));
                cmd.append_switch(Some(&"metrics-recording-only".into()));
                cmd.append_switch(Some(&"no-first-run".into()));
                // GPU memory optimization
                cmd.append_switch(Some(&"disable-gpu-shader-disk-cache".into()));
                cmd.append_switch_with_value(
                    Some(&"renderer-process-limit".into()),
                    Some(&"1".into()),
                );
                cmd.append_switch_with_value(
                    Some(&"disable-features".into()),
                    Some(&"AudioServiceOutOfProcess".into()),
                );
            }
        }

        fn render_process_handler(&self) -> Option<RenderProcessHandler> {
            Some(self.app.render_process_handler.clone())
        }
    }
}

pub fn initialize_cef() -> Result<()> {
    let _ = api_hash(cef_dll_sys::CEF_API_VERSION_LAST, 0);
    let args = Args::new();

    // Setup cache directory
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("nwidgets")
        .join("cef");
    let _ = std::fs::create_dir_all(&cache_dir);

    let render_handler = RenderProcessHandlerWrapper::new(GpuiRenderProcessHandler::new());

    let settings = Settings {
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        background_color: 0x00000000,
        uncaught_exception_stack_size: 0,
        root_cache_path: CefString::from(cache_dir.to_string_lossy().as_ref()),
        cache_path: CefString::from(cache_dir.to_string_lossy().as_ref()),
        log_severity: cef::LogSeverity::WARNING, // Show warnings and errors only
        ..Default::default()
    };
    let mut app = AppWrapper::new(CefAppStruct {
        render_process_handler: render_handler,
    });
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

#[allow(dead_code)]
pub fn shutdown_cef() {
    cef::shutdown();
}
