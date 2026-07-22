use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use std::time::Duration;
use tokio::fs;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemStats {
    pub cpu: u8,
    pub gpu: u8,
    pub ram: u8,
    pub cpu_temp: Option<u8>,
    pub gpu_temp: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct SystemStatsChanged;

pub struct SystemMonitorService {
    pub stats: SystemStats,
}

impl EventEmitter<SystemStatsChanged> for SystemMonitorService {}

struct GlobalSystemMonitorService(Entity<SystemMonitorService>);
impl Global for GlobalSystemMonitorService {}

impl SystemMonitorService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystemMonitorService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            stats: SystemStats::default(),
        });

        cx.set_global(GlobalSystemMonitorService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<SystemStats>();

        // Background system metrics collector thread
        gpui_tokio::Tokio::spawn(cx, async move {
            loop {
                let mut stats = SystemStats::default();

                // Read MemTotal & MemAvailable from /proc/meminfo
                if let Ok(meminfo) = fs::read_to_string("/proc/meminfo").await {
                    let mut total = 0u64;
                    let mut avail = 0u64;
                    for line in meminfo.lines() {
                        if line.starts_with("MemTotal:") {
                            total = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        } else if line.starts_with("MemAvailable:") {
                            avail = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        }
                    }
                    if total > 0 {
                        let used = total.saturating_sub(avail);
                        stats.ram = ((used as f32 / total as f32) * 100.0).round() as u8;
                    }
                }

                // Read CPU load estimation from /proc/stat
                if let Ok(stat) = fs::read_to_string("/proc/stat").await {
                    if let Some(first_line) = stat.lines().next() {
                        let parts: Vec<u64> = first_line
                            .split_whitespace()
                            .skip(1)
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if parts.len() >= 4 {
                            let idle = parts[3];
                            let total: u64 = parts.iter().sum();
                            let non_idle = total.saturating_sub(idle);
                            if total > 0 {
                                stats.cpu = ((non_idle as f32 / total as f32) * 100.0).round() as u8;
                            }
                        }
                    }
                }

                stats.gpu = (stats.cpu / 2).max(10); // GPU metric placeholder

                let _ = tx.unbounded_send(stats);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(new_stats) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.stats != new_stats {
                                srv.stats = new_stats;
                                cx.emit(SystemStatsChanged);
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
}
