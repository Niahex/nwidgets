use anyhow::Result;
use cef::{api_hash, args::Args, rc::Rc, App, CefString, ImplApp, ImplCommandLine, Settings, WrapApp};
use gpui::{App as GpuiApp, AsyncApp};
use std::time::Duration;

pub struct CefService;

impl gpui::Global for CefService {}

impl CefService {
    pub fn init(cx: &mut GpuiApp) {
        cx.set_global(CefService);

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
                cmd.append_switch(Some(&"disable-gpu".into()));
                cmd.append_switch(Some(&"disable-gpu-compositing".into()));
                cmd.append_switch(Some(&"enable-begin-frame-scheduling".into()));
                cmd.append_switch(Some(&"no-sandbox".into()));
                cmd.append_switch(Some(&"use-fake-ui-for-media-stream".into()));
                cmd.append_switch_with_value(
                    Some(&"ozone-platform".into()),
                    Some(&"wayland".into()),
                );
                // Use PipeWire for audio instead of ALSA/PulseAudio
                cmd.append_switch(Some(&"enable-features=WebRTCPipeWireCapturer".into()));
                cmd.append_switch_with_value(
                    Some(&"alsa-output-device".into()),
                    Some(&"pipewire".into()),
                );
                cmd.append_switch_with_value(
                    Some(&"alsa-input-device".into()),
                    Some(&"pipewire".into()),
                );
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
    let mut app = AppWrapper::new(CefAppStruct);
    let code = cef::execute_process(Some(args.as_main_args()), Some(&mut app), std::ptr::null_mut());
    if code >= 0 {
        std::process::exit(code);
    }
    let result = cef::initialize(Some(args.as_main_args()), Some(&settings), Some(&mut app), std::ptr::null_mut());
    if result != 1 {
        return Err(anyhow::anyhow!("Failed to initialize CEF"));
    }
    Ok(())
}

pub fn shutdown_cef() {
    cef::shutdown();
}
