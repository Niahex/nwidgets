use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct SystemMetric {
    pub name: SharedString,
    pub icon: &'static str,
    pub percent: u8,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SystemStats {
    pub cpu: u8,
    pub ram: u8,
    pub gpu: u8,
    pub network: u8,
    pub disk: u8,
}

impl SystemStats {
    pub fn metrics(&self) -> Vec<SystemMetric> {
        vec![
            SystemMetric {
                name: "CPU".into(),
                icon: "cpu",
                percent: self.cpu,
            },
            SystemMetric {
                name: "RAM".into(),
                icon: "memory",
                percent: self.ram,
            },
            SystemMetric {
                name: "GPU".into(),
                icon: "gpu",
                percent: self.gpu,
            },
            SystemMetric {
                name: "Network".into(),
                icon: "network-eternet-unsecure",
                percent: self.network,
            },
            SystemMetric {
                name: "Disk".into(),
                icon: "drive-harddisk",
                percent: self.disk,
            },
        ]
    }
}

#[derive(Clone)]
pub struct SystemStatsChanged;

pub struct SystemMonitorService {
    stats: Arc<RwLock<SystemStats>>,
}

impl EventEmitter<SystemStatsChanged> for SystemMonitorService {}

impl SystemMonitorService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stats = Arc::new(RwLock::new(SystemStats::default()));
        let stats_clone = Arc::clone(&stats);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<SystemStats>();

        gpui_tokio::Tokio::spawn(cx, async move { Self::monitor_worker(ui_tx).await }).detach();

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(new_stats) = ui_rx.next().await {
                    let changed = {
                        let mut current = stats_clone.write();
                        if *current != new_stats {
                            *current = new_stats;
                            true
                        } else {
                            false
                        }
                    };

                    if changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(SystemStatsChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { stats }
    }

    pub fn stats(&self) -> SystemStats {
        self.stats.read().clone()
    }

    async fn monitor_worker(ui_tx: futures::channel::mpsc::UnboundedSender<SystemStats>) {
        loop {
            let stats = SystemStats {
                cpu: Self::read_cpu().await,
                ram: Self::read_ram().await,
                gpu: Self::read_gpu().await,
                network: Self::read_network().await,
                disk: Self::read_disk().await,
            };

            let _ = ui_tx.unbounded_send(stats);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    async fn read_cpu() -> u8 {
        tokio::fs::read_to_string("/proc/stat")
            .await
            .ok()
            .and_then(|s| {
                let line = s.lines().next()?;
                let parts: Vec<_> = line.split_whitespace().skip(1).collect();
                if parts.len() >= 4 {
                    let user: u64 = parts[0].parse().ok()?;
                    let nice: u64 = parts[1].parse().ok()?;
                    let system: u64 = parts[2].parse().ok()?;
                    let idle: u64 = parts[3].parse().ok()?;
                    let total = user + nice + system + idle;
                    let used = user + nice + system;
                    Some(((used * 100) / total.max(1)) as u8)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    async fn read_ram() -> u8 {
        tokio::fs::read_to_string("/proc/meminfo")
            .await
            .ok()
            .and_then(|s| {
                let mut total = 0u64;
                let mut available = 0u64;
                for line in s.lines() {
                    if line.starts_with("MemTotal:") {
                        total = line.split_whitespace().nth(1)?.parse().ok()?;
                    } else if line.starts_with("MemAvailable:") {
                        available = line.split_whitespace().nth(1)?.parse().ok()?;
                    }
                }
                if total > 0 {
                    Some((((total - available) * 100) / total) as u8)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    async fn read_gpu() -> u8 {
        0
    }

    async fn read_network() -> u8 {
        0
    }

    async fn read_disk() -> u8 {
        tokio::fs::read_to_string("/proc/mounts")
            .await
            .ok()
            .and_then(|s| {
                for line in s.lines() {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[1] == "/" {
                        if let Ok(output) = std::process::Command::new("df")
                            .arg("--output=pcent")
                            .arg("/")
                            .output()
                        {
                            if let Ok(text) = String::from_utf8(output.stdout) {
                                if let Some(line) = text.lines().nth(1) {
                                    if let Ok(percent) = line.trim().trim_end_matches('%').parse::<u8>() {
                                        return Some(percent);
                                    }
                                }
                            }
                        }
                    }
                }
                None
            })
            .unwrap_or(0)
    }
}

struct GlobalSystemMonitorService(Entity<SystemMonitorService>);
impl Global for GlobalSystemMonitorService {}

impl SystemMonitorService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystemMonitorService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalSystemMonitorService(service.clone()));
        service
    }
}
