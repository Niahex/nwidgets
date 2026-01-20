use super::manager::{ConnectionType, NetworkManagerState};
use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct VpnConnection {
    pub name: SharedString,
    pub uuid: SharedString,
    pub connected: bool,
    pub vpn_type: SharedString,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VpnState {
    pub active: bool,
    pub connections: Vec<VpnConnection>,
}

#[derive(Clone)]
pub struct VpnStateChanged;

pub struct VpnService {
    state: Arc<RwLock<VpnState>>,
}

impl EventEmitter<VpnStateChanged> for VpnService {}

impl VpnService {
    pub fn new(
        cx: &mut Context<Self>,
        mut rx: futures::channel::mpsc::UnboundedReceiver<NetworkManagerState>,
    ) -> Self {
        let state = Arc::new(RwLock::new(VpnState::default()));
        let state_clone = Arc::clone(&state);

        let (list_tx, mut list_rx) = futures::channel::mpsc::unbounded::<Vec<VpnConnection>>();

        // Worker to list all VPN connections
        gpui_tokio::Tokio::spawn(cx, async move {
            loop {
                let connections = Self::list_vpn_connections().await;
                let _ = list_tx.unbounded_send(connections);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        })
        .detach();

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    tokio::select! {
                        Some(nm_state) = rx.next() => {
                            let active = nm_state.active_connections.iter().any(|c| c.conn_type == ConnectionType::Vpn);
                            let changed = {
                                let mut current = state_clone.write();
                                if current.active != active {
                                    current.active = active;
                                    true
                                } else {
                                    false
                                }
                            };

                            if changed {
                                let _ = this.update(&mut cx, |_, cx| {
                                    cx.emit(VpnStateChanged);
                                    cx.notify();
                                });
                            }
                        }
                        Some(connections) = list_rx.next() => {
                            let changed = {
                                let mut current = state_clone.write();
                                if current.connections != connections {
                                    current.connections = connections;
                                    true
                                } else {
                                    false
                                }
                            };

                            if changed {
                                let _ = this.update(&mut cx, |_, cx| {
                                    cx.emit(VpnStateChanged);
                                    cx.notify();
                                });
                            }
                        }
                    }
                }
            }
        })
        .detach();

        Self { state }
    }

    pub fn state(&self) -> VpnState {
        self.state.read().clone()
    }

    pub fn connect(&self, uuid: SharedString, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("nmcli")
                .args(["connection", "up", "uuid", &uuid])
                .output()
                .await;
        })
        .detach();
    }

    pub fn disconnect(&self, uuid: SharedString, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("nmcli")
                .args(["connection", "down", "uuid", &uuid])
                .output()
                .await;
        })
        .detach();
    }

    async fn list_vpn_connections() -> Vec<VpnConnection> {
        let mut connections = Vec::new();

        if let Ok(output) = tokio::process::Command::new("nmcli")
            .args(["-t", "-f", "NAME,UUID,TYPE,DEVICE", "connection", "show"])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                for line in text.lines() {
                    let parts: Vec<_> = line.split(':').collect();
                    if parts.len() >= 4 {
                        let vpn_type = parts[2];
                        if vpn_type.contains("vpn")
                            || vpn_type.contains("wireguard")
                            || vpn_type.contains("openvpn")
                        {
                            connections.push(VpnConnection {
                                name: parts[0].to_string().into(),
                                uuid: parts[1].to_string().into(),
                                connected: !parts[3].is_empty() && parts[3] != "--",
                                vpn_type: vpn_type.to_string().into(),
                            });
                        }
                    }
                }
            }
        }

        connections
    }
}

struct GlobalVpnService(Entity<VpnService>);
impl Global for GlobalVpnService {}

impl VpnService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalVpnService>().0.clone()
    }

    pub fn set_global(cx: &mut App, service: Entity<Self>) {
        cx.set_global(GlobalVpnService(service));
    }
}
