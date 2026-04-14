use super::item_client::fetch_item_data;
use super::types::{TrayItem, TrayItemAdded, TrayItemRemoved, TrayStateChanged};
use super::watcher::{start_watcher, WatcherEvent};
use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SystrayService {
    pub items: Arc<RwLock<Vec<TrayItem>>>,
}

impl EventEmitter<TrayItemAdded> for SystrayService {}
impl EventEmitter<TrayItemRemoved> for SystrayService {}
impl EventEmitter<TrayStateChanged> for SystrayService {}

impl SystrayService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let items = Arc::new(RwLock::new(Vec::new()));
        let (watcher_tx, mut watcher_rx) = mpsc::unbounded::<WatcherEvent>();
        let (ui_tx, mut ui_rx) = mpsc::unbounded::<UiEvent>();

        // Start D-Bus watcher service
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Err(e) = start_watcher(watcher_tx).await {
                log::error!("Failed to start StatusNotifierWatcher: {}", e);
            }
        })
        .detach();

        // Worker task - handles watcher events
        gpui_tokio::Tokio::spawn(cx, async move {
            while let Some(event) = watcher_rx.next().await {
                match event {
                    WatcherEvent::ItemRegistered(service) => {
                        log::info!("Item registered: {}", service);
                        
                        let (service_name, object_path) = parse_service_string(&service);
                        
                        match fetch_item_data(&service_name, &object_path).await {
                            Ok(item) => {
                                if let Err(e) = ui_tx.unbounded_send(UiEvent::ItemAdded(item)) {
                                    log::warn!("Failed to send systray item added event: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to fetch item data for {}: {}", service, e);
                            }
                        }
                    }
                    WatcherEvent::ItemUnregistered(service) => {
                        log::info!("Item unregistered: {}", service);
                        if let Err(e) = ui_tx.unbounded_send(UiEvent::ItemRemoved(service)) {
                            log::warn!("Failed to send systray item removed event: {}", e);
                        }
                    }
                    WatcherEvent::HostRegistered => {
                        log::info!("Host registered");
                    }
                }
            }
        })
        .detach();

        // UI task - updates state and emits events
        let items_clone = Arc::clone(&items);
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(event) = ui_rx.next().await {
                    match event {
                        UiEvent::ItemAdded(item) => {
                            let changed = {
                                let mut current_items = items_clone.write();
                                if !current_items.iter().any(|i: &TrayItem| i.service == item.service) {
                                    current_items.push(item.clone());
                                    true
                                } else {
                                    false
                                }
                            };

                            if changed {
                                let _ = this.update(&mut cx, |_, cx| {
                                    cx.emit(TrayItemAdded { item: item.clone() });
                                    cx.emit(TrayStateChanged);
                                    cx.notify();
                                });
                            }
                        }
                        UiEvent::ItemRemoved(service) => {
                            let changed = {
                                let mut current_items = items_clone.write();
                                let initial_len = current_items.len();
                                current_items.retain(|i| i.service != service);
                                current_items.len() != initial_len
                            };

                            if changed {
                                let _ = this.update(&mut cx, |_, cx| {
                                    cx.emit(TrayItemRemoved { service: service.clone() });
                                    cx.emit(TrayStateChanged);
                                    cx.notify();
                                });
                            }
                        }
                    }
                }
            }
        })
        .detach();

        Self { items }
    }
}

#[derive(Debug)]
enum UiEvent {
    ItemAdded(TrayItem),
    ItemRemoved(String),
}

fn parse_service_string(service: &str) -> (String, String) {
    if let Some(first_slash) = service.find('/') {
        let service_name = &service[..first_slash];
        let object_path = &service[first_slash..];
        (service_name.to_string(), object_path.to_string())
    } else {
        (service.to_string(), "/StatusNotifierItem".to_string())
    }
}

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
