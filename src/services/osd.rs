use crate::services::audio::{AudioService, AudioStateChanged};
use crate::services::clipboard::{ClipboardEvent, ClipboardMonitor};
use crate::services::lock_state::{LockMonitor, LockStateChanged, LockType};
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions, Layer};
use parking_lot::RwLock;
use std::sync::Arc;
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
    pub visible: bool,
}

#[derive(Clone)]
pub struct OsdService {
    current_event: Arc<RwLock<Option<OsdEvent>>>,
    visible: Arc<RwLock<bool>>,
    first_event: Arc<RwLock<bool>>,
    hide_task: Arc<RwLock<Option<Task<()>>>>,
    window_handle: Arc<RwLock<Option<AnyWindowHandle>>>,
    lock_monitor: Entity<LockMonitor>,
    clipboard_monitor: Entity<ClipboardMonitor>,
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        // Initialiser les moniteurs s'ils ne le sont pas déjà (ou créer des instances locales si ce ne sont pas des Singletons globaux)
        // Ici LockMonitor et ClipboardMonitor sont implémentés comme des Models locaux qu'on instancie.
        let lock_monitor = LockMonitor::init(cx);
        let clipboard_monitor = ClipboardMonitor::init(cx);

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

        // Abonnement CapsLock/NumLock
        cx.subscribe(&lock_monitor, |this, _monitor, event: &LockStateChanged, cx| {
            if let LockType::CapsLock = event.lock_type {
                this.show_event(OsdEvent::CapsLock(event.enabled), cx);
            }
        })
        .detach();

        // Abonnement Clipboard
        cx.subscribe(&clipboard_monitor, |this, _monitor, _event: &ClipboardEvent, cx| {
            this.show_event(OsdEvent::Clipboard, cx);
        })
        .detach();

        Self {
            current_event: Arc::new(RwLock::new(None)),
            visible: Arc::new(RwLock::new(false)),
            first_event,
            hide_task: Arc::new(RwLock::new(None)),
            window_handle: Arc::new(RwLock::new(None)),
            lock_monitor,
            clipboard_monitor,
        }
    }

    pub fn show_event(&self, event: OsdEvent, cx: &mut Context<Self>) {
        *self.current_event.write() = Some(event.clone());
        *self.visible.write() = true;

        // Annuler le timer précédent s'il existe
        *self.hide_task.write() = None;

        // Ouvrir la fenêtre si elle n'existe pas via une tâche asynchrone
        // On clone le service (struct légère avec Arc) pour l'utiliser sans verrouiller l'entité GPUI
        let service = self.clone();
        cx.spawn(|_, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                cx.update(|cx| {
                    service.open_window(cx);
                })
                .ok();
            }
        })
        .detach();

        cx.emit(OsdStateChanged {
            event: Some(event),
            visible: true,
        });

        // Lancer un nouveau timer pour cacher dans 2.5s
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2500))
                .await;

            // Correction ici: passer `cx` directement (par valeur/move), pas &mut cx
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

        // Fermer la fenêtre
        self.close_window(cx);

        cx.emit(OsdStateChanged {
            event: None,
            visible: false,
        });
        cx.notify();
    }

    fn open_window(&self, cx: &mut App) {
        if self.window_handle.read().is_some() {
            return; // Déjà ouverte
        }

        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px((3440.0 - 400.0) / 2.0),
                        y: px(1440.0 - 64.0 - 80.0),
                    },
                    size: Size {
                        width: px(400.0),
                        height: px(64.0),
                    },
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
            |_window, cx| {
                use crate::widgets::osd::OsdWidget;
                cx.new(OsdWidget::new)
            },
        );

        if let Ok(handle) = handle {
            *self.window_handle.write() = Some(handle.into());
        }
    }

    fn close_window(&self, cx: &mut Context<Self>) {
        if let Some(handle) = self.window_handle.write().take() {
            let _ = handle.update(cx, |_, window, _| {
                window.remove_window();
            });
        }
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
        let service = cx.new(Self::new);
        cx.set_global(GlobalOsdService(service.clone()));
        service
    }
}