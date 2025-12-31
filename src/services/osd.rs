use crate::services::audio::{AudioService, AudioStateChanged};
use crate::services::clipboard::{ClipboardEvent, ClipboardMonitor};
use crate::services::control_center::ControlCenterService;
use crate::services::lock_state::{LockMonitor, LockStateChanged, LockType};
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions, Layer};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(String, u8, bool), // icon_name, volume %, muted
    Microphone(bool),         // muted
    CapsLock(bool),           // enabled
    Clipboard,                // copied
}

#[derive(Clone)]
pub struct OsdStateChanged {
    pub event: Option<OsdEvent>,
}

pub struct OsdService {
    current_event: Option<OsdEvent>,
    window_handle: Option<AnyWindowHandle>,
    hide_task: Option<Task<()>>,
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let lock_monitor = LockMonitor::init(cx);
        let clipboard_monitor = ClipboardMonitor::init(cx);

        let mut first_event = true;

        cx.subscribe(&audio, move |this, _audio, event: &AudioStateChanged, cx| {
            if first_event {
                first_event = false;
                return;
            }
            
            let control_center = ControlCenterService::global(cx);
            if control_center.read(cx).is_visible() {
                return;
            }

            let state = &event.state;
            let icon_name = if state.sink_muted {
                "sink-muted".to_string()
            } else if state.sink_volume == 0 {
                "sink-zero".to_string()
            } else if state.sink_volume < 33 {
                "sink-low".to_string()
            } else if state.sink_volume < 66 {
                "sink-medium".to_string()
            } else {
                "sink-high".to_string()
            };

            this.show_event(OsdEvent::Volume(icon_name, state.sink_volume, state.sink_muted), cx);
        }).detach();

        cx.subscribe(&lock_monitor, |this, _monitor, event: &LockStateChanged, cx| {
            if let LockType::CapsLock = event.lock_type {
                this.show_event(OsdEvent::CapsLock(event.enabled), cx);
            }
        }).detach();

        cx.subscribe(&clipboard_monitor, |this, _monitor, _event: &ClipboardEvent, cx| {
            this.show_event(OsdEvent::Clipboard, cx);
        }).detach();

        Self {
            current_event: None,
            window_handle: None,
            hide_task: None,
        }
    }

    pub fn show_event(&mut self, event: OsdEvent, cx: &mut Context<Self>) {
        self.current_event = Some(event.clone());
        self.hide_task = None;

        if self.window_handle.is_none() {
            self.open_window(cx);
        }

        cx.emit(OsdStateChanged {
            event: Some(event),
        });

        let task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(2500)).await;
            this.update(cx, |service, cx| service.hide(cx)).ok();
        });

        self.hide_task = Some(task);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.current_event = None;
        self.hide_task = None;
        cx.emit(OsdStateChanged { event: None });
        cx.notify();
    }

    fn open_window(&mut self, cx: &mut Context<Self>) {
        use crate::widgets::osd::OsdWidget;
        
        let bounds = cx.displays().first().unwrap().bounds();
        let width = px(400.0);
        let height = px(64.0);
        
        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: bounds.center().x - width / 2.0,
                        y: bounds.bottom() - height - px(80.0),
                    },
                    size: Size { width, height },
                })),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-osd".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::BOTTOM,
                    exclusive_zone: None,
                    margin: Some((px(0.0), px(0.0), px(80.0), px(0.0))),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(OsdWidget::new),
        );

        if let Ok(handle) = handle {
            self.window_handle = Some(handle.into());
        }
    }

    pub fn current_event(&self) -> Option<OsdEvent> {
        self.current_event.clone()
    }

    pub fn is_visible(&self) -> bool {
        self.window_handle.is_some()
    }
}

// Global accessor
struct GlobalOsdService(Entity<OsdService>);
impl Global for GlobalOsdService {}

impl OsdService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalOsdService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalOsdService(service.clone()));
        service
    }
}