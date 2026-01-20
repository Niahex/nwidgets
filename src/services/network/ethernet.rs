use super::manager::{ConnectionType, NetworkManagerState};
use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EthernetState {
    pub connected: bool,
    pub interface_name: Option<SharedString>,
}

#[derive(Clone)]
pub struct EthernetStateChanged;

pub struct EthernetService {
    state: Arc<RwLock<EthernetState>>,
}

impl EventEmitter<EthernetStateChanged> for EthernetService {}

impl EthernetService {
    pub fn new(
        cx: &mut Context<Self>,
        mut rx: futures::channel::mpsc::UnboundedReceiver<NetworkManagerState>,
    ) -> Self {
        let state = Arc::new(RwLock::new(EthernetState::default()));
        let state_clone = Arc::clone(&state);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(nm_state) = rx.next().await {
                    let new_state = Self::extract_ethernet_state(&nm_state);
                    let changed = {
                        let mut current = state_clone.write();
                        if *current != new_state {
                            *current = new_state;
                            true
                        } else {
                            false
                        }
                    };

                    if changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(EthernetStateChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { state }
    }

    pub fn state(&self) -> EthernetState {
        self.state.read().clone()
    }

    fn extract_ethernet_state(nm_state: &NetworkManagerState) -> EthernetState {
        let ethernet_conn = nm_state
            .active_connections
            .iter()
            .find(|c| c.conn_type == ConnectionType::Ethernet);

        if let Some(conn) = ethernet_conn {
            EthernetState {
                connected: true,
                interface_name: Some(conn.id.clone().into()),
            }
        } else {
            EthernetState::default()
        }
    }
}

struct GlobalEthernetService(Entity<EthernetService>);
impl Global for GlobalEthernetService {}

impl EthernetService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalEthernetService>().0.clone()
    }

    pub fn set_global(cx: &mut App, service: Entity<Self>) {
        cx.set_global(GlobalEthernetService(service));
    }
}
