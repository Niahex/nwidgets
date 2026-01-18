use crate::services::audio::{AudioService, AudioStateChanged};
use crate::services::clipboard::{ClipboardEvent, ClipboardMonitor};
use crate::services::lock_state::{LockMonitor, LockStateChanged};
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;
use std::time::Duration;

// Constantes pour éviter les allocations répétées de strings
const SINK_MUTED: &str = "sink-muted";
const SINK_ZERO: &str = "sink-zero";
const SINK_LOW: &str = "sink-low";
const SINK_MEDIUM: &str = "sink-medium";
const SINK_HIGH: &str = "sink-high";

#[derive(Debug, Clone, PartialEq)]
pub enum OsdEvent {
    Volume(String, u8, bool), // icon_name, volume %, muted
    #[allow(dead_code)]
    Microphone(bool), // muted
    CapsLock(bool),           // enabled
    Clipboard,                // copied
}

#[derive(Clone)]
pub struct OsdStateChanged {
    pub event: Option<OsdEvent>,
    pub visible: bool,
}

pub struct OsdService {
    current_event: Option<OsdEvent>,
    visible: bool,
    window_handle: Option<AnyWindowHandle>,
    hide_task: Option<Task<()>>,
    _lock_monitor: Entity<LockMonitor>, // Garder le LockMonitor en vie
    _clipboard_monitor: Entity<ClipboardMonitor>, // Garder le ClipboardMonitor en vie
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let lock_monitor = LockMonitor::init(cx);
        let clipboard_monitor = ClipboardMonitor::init(cx);

        // Écouter les changements audio
        cx.subscribe(
            &audio,
            move |this, _audio, event: &AudioStateChanged, cx| {
                let state = &event.state;
                let icon_name = if state.sink_muted {
                    SINK_MUTED
                } else if state.sink_volume == 0 {
                    SINK_ZERO
                } else if state.sink_volume < 33 {
                    SINK_LOW
                } else if state.sink_volume < 66 {
                    SINK_MEDIUM
                } else {
                    SINK_HIGH
                };

                this.show_event(
                    OsdEvent::Volume(icon_name.to_string(), state.sink_volume, state.sink_muted),
                    cx,
                );
            },
        )
        .detach();

        cx.subscribe(
            &lock_monitor,
            |this, _monitor, event: &LockStateChanged, cx| {
                this.show_event(OsdEvent::CapsLock(event.enabled), cx);
            },
        )
        .detach();

        cx.subscribe(
            &clipboard_monitor,
            |this, _monitor, _event: &ClipboardEvent, cx| {
                this.show_event(OsdEvent::Clipboard, cx);
            },
        )
        .detach();

        let mut this = Self {
            current_event: None,
            visible: false,
            window_handle: None,
            hide_task: None,
            _lock_monitor: lock_monitor,
            _clipboard_monitor: clipboard_monitor,
        };

        this.open_window(cx);
        this
    }

    pub fn show_event(&mut self, event: OsdEvent, cx: &mut Context<Self>) {
        // Ensure window exists
        if self.window_handle.is_none() {
            self.open_window(cx);
        }

        // Cancel previous hide task
        self.hide_task = None;

        let mut changed = false;

        // Update data if changed
        if self.current_event.as_ref() != Some(&event) {
            self.current_event = Some(event.clone());
            changed = true;
        }

        // Make visible if not already
        if !self.visible {
            self.visible = true;
            changed = true;
            
            // Restaurer vers la layer Overlay quand visible
            if let Some(window_handle) = &self.window_handle {
                let _ = window_handle.update(cx, |_, window, _| {
                    window.set_layer(gpui::layer_shell::Layer::Overlay);
                });
            }
        }

        if changed {
            cx.emit(OsdStateChanged {
                event: Some(event),
                visible: true,
            });
            cx.notify();
        }

        // Restart hide timer (debounce)
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1500))
                .await;
            this.update(cx, |service, cx| service.hide(cx)).ok();
        });

        self.hide_task = Some(task);
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        if self.visible {
            self.visible = false;
            
            // Déplacer vers la layer Background quand caché
            if let Some(window_handle) = &self.window_handle {
                let _ = window_handle.update(cx, |_, window, _| {
                    window.set_layer(gpui::layer_shell::Layer::Background);
                });
            }
            
            // Note: We do NOT clear current_event here.
            // We keep the data so the UI can fade it out gracefully.
            cx.emit(OsdStateChanged {
                event: self.current_event.clone(),
                visible: false,
            });
            cx.notify();
        }
        self.hide_task = None;
    }

    fn open_window(&mut self, cx: &mut Context<Self>) {
        use crate::widgets::osd::OsdWidget;

        let displays = cx.displays();
        let Some(display) = displays.first() else {
            return;
        };
        let bounds = display.bounds();
        let width = px(400.0);
        let height = px(64.0);

        // Capture state to pass to widget constructor
        let initial_event = self.current_event.clone();
        let initial_visible = self.visible;

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
            move |_, cx| cx.new(|cx| OsdWidget::new(cx, initial_event, initial_visible)),
        );

        if let Ok(handle) = handle {
            self.window_handle = Some(handle.into());
            
            // L'OSD n'a jamais besoin de recevoir des clics, désactiver complètement
            let _ = handle.update(cx, |_, window, _| {
                window.set_input_region(None);
            });
        }
    }

    #[allow(dead_code)]
    pub fn current_event(&self) -> Option<OsdEvent> {
        self.current_event.clone()
    }

    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible
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
