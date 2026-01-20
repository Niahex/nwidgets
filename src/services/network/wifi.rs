use super::manager::{AccessPointProxy, ConnectionType, NetworkManagerState};
use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use zbus::Connection;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WifiState {
    pub connected: bool,
    pub ssid: Option<SharedString>,
    pub signal_strength: u8,
}

#[derive(Clone)]
pub struct WifiStateChanged;

pub struct WifiService {
    state: Arc<RwLock<WifiState>>,
}

impl EventEmitter<WifiStateChanged> for WifiService {}

impl WifiService {
    pub fn new(
        cx: &mut Context<Self>,
        mut rx: futures::channel::mpsc::UnboundedReceiver<NetworkManagerState>,
    ) -> Self {
        let state = Arc::new(RwLock::new(WifiState::default()));
        let state_clone = Arc::clone(&state);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(nm_state) = rx.next().await {
                    let new_state = Self::extract_wifi_state(&nm_state).await;
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
                            cx.emit(WifiStateChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { state }
    }

    pub fn state(&self) -> WifiState {
        self.state.read().clone()
    }

    async fn extract_wifi_state(nm_state: &NetworkManagerState) -> WifiState {
        let wifi_conn = nm_state
            .active_connections
            .iter()
            .find(|c| c.conn_type == ConnectionType::Wifi);

        if let Some(conn) = wifi_conn {
            let mut state = WifiState {
                connected: true,
                ssid: Some(conn.id.clone().into()),
                signal_strength: 0,
            };

            // Get signal strength
            if conn.specific_object.as_str() != "/" {
                if let Ok(system_conn) = Connection::system().await {
                    if let Ok(ap) = AccessPointProxy::new(&system_conn, conn.specific_object.clone()).await {
                        if let Ok(strength) = ap.strength().await {
                            state.signal_strength = strength;
                        }
                    }
                }
            }

            state
        } else {
            WifiState::default()
        }
    }
}

struct GlobalWifiService(Entity<WifiService>);
impl Global for GlobalWifiService {}

impl WifiService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalWifiService>().0.clone()
    }

    pub fn set_global(cx: &mut App, service: Entity<Self>) {
        cx.set_global(GlobalWifiService(service));
    }
}
