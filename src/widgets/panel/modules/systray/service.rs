use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::sync::Arc;
use system_tray::client::{Client, Event, UpdateEvent};

#[derive(Clone, Debug, PartialEq)]
pub struct SystrayItem {
    pub id: SharedString,
    pub title: SharedString,
    pub icon_name: Option<SharedString>,
    pub address: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SystrayState {
    pub items: SmallVec<[SystrayItem; 8]>,
}

#[derive(Clone)]
pub struct SystrayChanged;

pub struct SystrayService {
    state: Arc<RwLock<SystrayState>>,
    client: Arc<RwLock<Option<Client>>>,
}

impl EventEmitter<SystrayChanged> for SystrayService {}

impl SystrayService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(SystrayState::default()));
        let state_clone = Arc::clone(&state);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<SystrayState>();

        let client = Arc::new(RwLock::new(None));
        let client_clone = Arc::clone(&client);

        // Worker Task (Tokio) - uses system-tray crate
        gpui_tokio::Tokio::spawn(cx, async move {
            Self::systray_worker(ui_tx, client_clone).await
        })
        .detach();

        // UI Task (GPUI)
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(new_state) = ui_rx.next().await {
                    let state_changed = {
                        let mut current_state = state_clone.write();
                        if *current_state != new_state {
                            *current_state = new_state;
                            true
                        } else {
                            false
                        }
                    };

                    if state_changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(SystrayChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { state, client }
    }

    pub fn items(&self) -> Vec<SystrayItem> {
        self.state.read().items.to_vec()
    }

    pub fn state(&self) -> SystrayState {
        self.state.read().clone()
    }

    async fn systray_worker(
        ui_tx: futures::channel::mpsc::UnboundedSender<SystrayState>,
        client_arc: Arc<RwLock<Option<Client>>>,
    ) {
        log::info!("Systray worker started");

        let client = match Client::new().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to create system-tray client: {e}");
                return;
            }
        };

        // Client is not cloneable, just use it directly

        let mut tray_rx = client.subscribe();

        // Send initial items
        let initial_state = Self::items_to_state(&client);
        log::info!("Found {} initial systray items", initial_state.items.len());
        let _ = ui_tx.unbounded_send(initial_state);

        // Listen for events
        while let Ok(event) = tray_rx.recv().await {
            log::info!("Systray event: {:?}", event);
            
            match event {
                Event::Add(address, item) => {
                    log::info!("Systray item added: {} - {:?}", address, item.id);
                    let new_state = Self::items_to_state(&client);
                    let _ = ui_tx.unbounded_send(new_state);
                }
                Event::Remove(address) => {
                    log::info!("Systray item removed: {}", address);
                    let new_state = Self::items_to_state(&client);
                    let _ = ui_tx.unbounded_send(new_state);
                }
                Event::Update(address, update) => {
                    log::info!("Systray item updated: {} - {:?}", address, update);
                    let new_state = Self::items_to_state(&client);
                    let _ = ui_tx.unbounded_send(new_state);
                }
            }
        }
    }

    fn items_to_state(client: &Client) -> SystrayState {
        let mut state_items = SmallVec::new();

        let items_map = client.items();
        let items_lock = items_map.lock().expect("Failed to lock items map");

        for (address, (item, _menu)) in items_lock.iter() {
            let id = item.id.clone();
            let title = item.title.clone().unwrap_or_else(|| id.clone());
            let icon_name = item.icon_name.clone();

            state_items.push(SystrayItem {
                id: id.into(),
                title: title.into(),
                icon_name: icon_name.map(|s| s.into()),
                address: address.clone(),
            });
        }

        SystrayState { items: state_items }
    }

    pub async fn activate_item(address: &str) {
        log::info!("Activating systray item: {}", address);
        
        let client = match Client::new().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to create system-tray client: {e}");
                return;
            }
        };

        let items_map = client.items();
        let exists = {
            let items_lock = items_map.lock().expect("Failed to lock items map");
            items_lock.contains_key(address)
        };
        if exists {
            if let Err(e) = client
                .activate(system_tray::client::ActivateRequest::Default {
                    address: address.to_string(),
                    x: 0,
                    y: 0,
                })
                .await
            {
                log::error!("Failed to activate systray item: {e}");
            }
        } else {
            log::warn!("Systray item not found: {}", address);
        }
    }
}

// Global accessor
struct GlobalSystrayService(Entity<SystrayService>);
impl Global for GlobalSystrayService {}

impl SystrayService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystrayService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalSystrayService(service.clone()));
        service
    }
}
