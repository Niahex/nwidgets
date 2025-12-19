use crate::services::audio::{AudioService, AudioStateChanged};
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(String, u8, bool), // icon_name, volume %, muted
    Microphone(bool),         // muted
}

#[derive(Clone)]
pub struct OsdStateChanged {
    pub event: Option<OsdEvent>,
    pub visible: bool,
}

pub struct OsdService {
    current_event: Arc<RwLock<Option<OsdEvent>>>,
    visible: Arc<RwLock<bool>>,
    first_event: Arc<RwLock<bool>>,
    hide_task: Arc<RwLock<Option<Task<()>>>>,
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let first_event = Arc::new(RwLock::new(true));
        let first_event_clone = Arc::clone(&first_event);

        // S'abonner aux changements audio pour afficher l'OSD automatiquement
        cx.subscribe(
            &audio,
            move |this, _audio, event: &AudioStateChanged, cx| {
                // Ignorer le premier événement (état initial)
                if *first_event_clone.read() {
                    *first_event_clone.write() = false;
                    return;
                }

                let state = &event.state;

                // Déterminer l'icône en fonction du volume et du mute
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

                // Afficher l'OSD pour le volume
                this.show_event(
                    OsdEvent::Volume(icon_name, state.sink_volume, state.sink_muted),
                    cx,
                );
            },
        )
        .detach();

        Self {
            current_event: Arc::new(RwLock::new(None)),
            visible: Arc::new(RwLock::new(false)),
            first_event,
            hide_task: Arc::new(RwLock::new(None)),
        }
    }

    pub fn show_event(&self, event: OsdEvent, cx: &mut Context<Self>) {
        *self.current_event.write() = Some(event.clone());
        *self.visible.write() = true;

        // Annuler le timer précédent s'il existe
        *self.hide_task.write() = None;

        cx.emit(OsdStateChanged {
            event: Some(event),
            visible: true,
        });

        // Lancer un nouveau timer pour cacher dans 2.5s
        let task = cx.spawn(async move |this, mut cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2500))
                .await;

            this.update(cx, |service, cx| {
                service.hide(cx);
            })
            .ok();
        });

        *self.hide_task.write() = Some(task);
        cx.notify();
    }

    pub fn hide(&self, cx: &mut Context<Self>) {
        *self.visible.write() = false;
        *self.hide_task.write() = None;

        cx.emit(OsdStateChanged {
            event: None,
            visible: false,
        });
        cx.notify();
    }

    pub fn current_event(&self) -> Option<OsdEvent> {
        self.current_event.read().clone()
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.read()
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
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalOsdService(service.clone()));
        service
    }
}
