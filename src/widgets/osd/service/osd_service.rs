use crate::services::media::audio::{AudioService, AudioStateChanged};
use crate::services::system::clipboard::{ClipboardEvent, ClipboardMonitor};
use crate::widgets::osd::monitors::{LockMonitor, LockStateChanged};
use crate::widgets::osd::types::{OsdEvent, OsdStateChanged, SINK_HIGH, SINK_LOW, SINK_MEDIUM, SINK_MUTED, SINK_ZERO, OSD_DISPLAY_DURATION_MS};
use crate::widgets::osd::window::OsdWindowManager;
use gpui::*;
use std::time::Duration;

pub struct OsdService {
    current_event: Option<OsdEvent>,
    visible: bool,
    window_manager: OsdWindowManager,
    hide_task: Option<Task<()>>,
    _lock_monitor: Entity<LockMonitor>,
    _clipboard_monitor: Entity<ClipboardMonitor>,
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let lock_monitor = LockMonitor::init(cx);
        let clipboard_monitor = ClipboardMonitor::init(cx);

        cx.subscribe(
            &audio,
            move |this, _audio, event: &AudioStateChanged, cx| {
                let state = &event.state;
                let icon_name = Self::get_volume_icon(state.sink_volume, state.sink_muted);
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

        Self {
            current_event: None,
            visible: false,
            window_manager: OsdWindowManager::new(),
            hide_task: None,
            _lock_monitor: lock_monitor,
            _clipboard_monitor: clipboard_monitor,
        }
    }

    fn get_volume_icon(volume: u8, muted: bool) -> &'static str {
        if muted {
            SINK_MUTED
        } else if volume == 0 {
            SINK_ZERO
        } else if volume < 33 {
            SINK_LOW
        } else if volume < 66 {
            SINK_MEDIUM
        } else {
            SINK_HIGH
        }
    }

    pub fn show_event(&mut self, event: OsdEvent, cx: &mut Context<Self>) {
        if !self.window_manager.has_window() {
            self.window_manager.open_window(cx);
        }

        self.hide_task = None;

        let mut changed = false;

        if self.current_event.as_ref() != Some(&event) {
            self.current_event = Some(event.clone());
            changed = true;
        }

        if !self.visible {
            self.visible = true;
            changed = true;
            self.window_manager.show_window(cx);
        }

        if changed {
            cx.emit(OsdStateChanged {
                event: Some(event),
                visible: true,
            });
            cx.notify();
        }

        let task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(OSD_DISPLAY_DURATION_MS))
                .await;
            let _ = this.update(cx, |service, cx| service.hide(cx));
        });

        self.hide_task = Some(task);
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        if !self.visible {
            return;
        }

        self.visible = false;
        self.window_manager.hide_window(cx);

        cx.emit(OsdStateChanged {
            event: None,
            visible: false,
        });
        cx.notify();
    }
}

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
