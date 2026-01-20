mod manager;
mod ethernet;
mod wifi;
mod vpn;

pub use ethernet::{EthernetService, EthernetState, EthernetStateChanged};
pub use wifi::{WifiService, WifiState, WifiStateChanged};
pub use vpn::{VpnService, VpnState, VpnStateChanged};

use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, Context, Entity, EventEmitter, Global};
use manager::NetworkManagerWorker;
use parking_lot::RwLock;
use std::sync::Arc;

// Unified NetworkState for backward compatibility
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetworkState {
    pub ethernet: EthernetState,
    pub wifi: WifiState,
    pub vpn: VpnState,
}

impl NetworkState {
    pub fn get_icon_name(&self) -> &'static str {
        if self.vpn.active {
            if self.wifi.connected {
                let signal_level = if self.wifi.signal_strength > 75 {
                    "high"
                } else if self.wifi.signal_strength > 50 {
                    "good"
                } else if self.wifi.signal_strength > 25 {
                    "medium"
                } else {
                    "low"
                };
                match signal_level {
                    "high" => "network-wifi-high-secure",
                    "good" => "network-wifi-good-secure",
                    "medium" => "network-wifi-medium-secure",
                    _ => "network-wifi-low-secure",
                }
            } else if self.ethernet.connected {
                "network-eternet-secure"
            } else {
                "network-eternet-disconnected"
            }
        } else if self.wifi.connected {
            let signal_level = if self.wifi.signal_strength > 75 {
                "high"
            } else if self.wifi.signal_strength > 50 {
                "good"
            } else if self.wifi.signal_strength > 25 {
                "medium"
            } else {
                "low"
            };
            match signal_level {
                "high" => "network-wifi-high-unsecure",
                "good" => "network-wifi-good-unsecure",
                "medium" => "network-wifi-medium-unsecure",
                _ => "network-wifi-low-unsecure",
            }
        } else if self.ethernet.connected {
            "network-eternet-unsecure"
        } else {
            "network-eternet-disconnected"
        }
    }

    pub fn ssid(&self) -> Option<gpui::SharedString> {
        self.wifi.ssid.clone()
    }
}

#[derive(Clone)]
pub struct NetworkStateChanged;

// Unified NetworkService that aggregates all network services
pub struct NetworkService {
    state: Arc<RwLock<NetworkState>>,
    ethernet: Entity<EthernetService>,
    wifi: Entity<WifiService>,
    vpn: Entity<VpnService>,
}

impl EventEmitter<NetworkStateChanged> for NetworkService {}

impl NetworkService {
    fn new(
        cx: &mut Context<Self>,
        ethernet: Entity<EthernetService>,
        wifi: Entity<WifiService>,
        vpn: Entity<VpnService>,
    ) -> Self {
        let state = Arc::new(RwLock::new(NetworkState::default()));

        // Subscribe to all sub-services
        cx.subscribe(&ethernet, |this, _, _: &EthernetStateChanged, cx| {
            this.update_state(cx);
        })
        .detach();

        cx.subscribe(&wifi, |this, _, _: &WifiStateChanged, cx| {
            this.update_state(cx);
        })
        .detach();

        cx.subscribe(&vpn, |this, _, _: &VpnStateChanged, cx| {
            this.update_state(cx);
        })
        .detach();

        let service = Self {
            state,
            ethernet,
            wifi,
            vpn,
        };

        // Initial state update
        service.update_state_internal(cx);

        service
    }

    fn update_state(&self, cx: &mut Context<Self>) {
        self.update_state_internal(cx);
        cx.emit(NetworkStateChanged);
        cx.notify();
    }

    fn update_state_internal(&self, cx: &App) {
        let mut state = self.state.write();
        state.ethernet = self.ethernet.read(cx).state();
        state.wifi = self.wifi.read(cx).state();
        state.vpn = self.vpn.read(cx).state();
    }

    pub fn state(&self) -> NetworkState {
        self.state.read().clone()
    }

    pub fn ethernet(&self) -> Entity<EthernetService> {
        self.ethernet.clone()
    }

    pub fn wifi(&self) -> Entity<WifiService> {
        self.wifi.clone()
    }

    pub fn vpn(&self) -> Entity<VpnService> {
        self.vpn.clone()
    }
}

struct GlobalNetworkService(Entity<NetworkService>);
impl Global for GlobalNetworkService {}

impl NetworkService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNetworkService>().0.clone()
    }
}

/// Initialize all network services with a shared NetworkManager worker
pub fn init_network_services(cx: &mut App) {
    let worker = NetworkManagerWorker::new();
    
    // Create broadcast channels
    let (tx1, rx1) = futures::channel::mpsc::unbounded();
    let (tx2, rx2) = futures::channel::mpsc::unbounded();
    let (tx3, rx3) = futures::channel::mpsc::unbounded();
    
    // Spawn the shared worker
    gpui_tokio::Tokio::spawn(cx, async move {
        let (main_tx, mut main_rx) = futures::channel::mpsc::unbounded();
        
        // Spawn worker
        tokio::spawn(async move {
            worker.run(main_tx).await;
        });
        
        // Broadcast to all services
        while let Some(state) = main_rx.next().await {
            let _ = tx1.unbounded_send(state.clone());
            let _ = tx2.unbounded_send(state.clone());
            let _ = tx3.unbounded_send(state);
        }
    }).detach();
    
    // Initialize sub-services
    let ethernet = cx.new(|cx| EthernetService::new(cx, rx1));
    EthernetService::set_global(cx, ethernet.clone());
    
    let wifi = cx.new(|cx| WifiService::new(cx, rx2));
    WifiService::set_global(cx, wifi.clone());
    
    let vpn = cx.new(|cx| VpnService::new(cx, rx3));
    VpnService::set_global(cx, vpn.clone());

    // Initialize unified service
    let network = cx.new(|cx| NetworkService::new(cx, ethernet, wifi, vpn));
    cx.set_global(GlobalNetworkService(network));
}
