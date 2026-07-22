use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkState {
    pub wifi_enabled: bool,
    pub active_ssid: Option<String>,
    pub networks: Vec<WifiNetwork>,
}

#[derive(Debug, Clone)]
pub struct NetworkStateChanged;

pub struct NetworkService {
    pub state: NetworkState,
}

impl EventEmitter<NetworkStateChanged> for NetworkService {}

struct GlobalNetworkService(Entity<NetworkService>);
impl Global for GlobalNetworkService {}

impl NetworkService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNetworkService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            state: NetworkState::default(),
        });

        cx.set_global(GlobalNetworkService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<NetworkState>();

        // Background worker to query nmcli
        gpui_tokio::Tokio::spawn(cx, async move {
            let mut state = NetworkState::default();

            if let Ok(out) = Command::new("nmcli").args(["radio", "wifi"]).output().await {
                let s = String::from_utf8_lossy(&out.stdout);
                state.wifi_enabled = s.trim() == "enabled";
            }

            if let Ok(out) = Command::new("nmcli").args(["-t", "-f", "IN-USE,SSID,SIGNAL", "dev", "wifi"]).output().await {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 {
                        let active = parts[0] == "*";
                        let ssid = parts[1].to_string();
                        let signal = parts[2].parse::<u8>().unwrap_or(0);
                        if !ssid.is_empty() {
                            if active {
                                state.active_ssid = Some(ssid.clone());
                            }
                            state.networks.push(WifiNetwork {
                                ssid,
                                signal,
                                active,
                            });
                        }
                    }
                }
            }

            let _ = tx.unbounded_send(state);
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(new_state) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.state != new_state {
                                srv.state = new_state;
                                cx.emit(NetworkStateChanged);
                                cx.notify();
                            }
                        });
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn toggle_wifi(&mut self, cx: &mut Context<Self>) {
        self.state.wifi_enabled = !self.state.wifi_enabled;
        cx.notify();
        let target_state = if self.state.wifi_enabled { "on" } else { "off" };
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("nmcli")
                .args(["radio", "wifi", target_state])
                .status()
                .await;
        })
        .detach();
    }
}
