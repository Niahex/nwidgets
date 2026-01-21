use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct DiskInfo {
    pub name: SharedString,
    pub mount: SharedString,
    pub percent: u8,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SystemStats {
    pub cpu: u8,
    pub cpu_temp: Option<f32>,
    pub ram: u8,
    pub gpu: u8,
    pub gpu_temp: Option<f32>,
    pub disks: Vec<DiskInfo>,
    pub net_up: u64,
    pub net_down: u64,
    pub net_total: u64,
}

#[derive(Clone)]
pub struct SystemStatsChanged;

pub struct SystemMonitorService {
    stats: Arc<RwLock<SystemStats>>,
    monitoring_enabled: Arc<RwLock<bool>>,
    notify: Arc<tokio::sync::Notify>,
}

impl EventEmitter<SystemStatsChanged> for SystemMonitorService {}

impl SystemMonitorService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stats = Arc::new(RwLock::new(SystemStats::default()));
        let stats_clone = Arc::clone(&stats);
        let monitoring_enabled = Arc::new(RwLock::new(false));
        let monitoring_enabled_clone = Arc::clone(&monitoring_enabled);
        let notify = Arc::new(tokio::sync::Notify::new());
        let notify_clone = Arc::clone(&notify);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<SystemStats>();

        gpui_tokio::Tokio::spawn(cx, async move {
            Self::monitor_worker(ui_tx, monitoring_enabled_clone, notify_clone).await
        })
        .detach();

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

        Self {
            stats,
            monitoring_enabled,
            notify,
        }
    }

    pub fn stats(&self) -> SystemStats {
        self.stats.read().clone()
    }

    pub fn enable_monitoring(&self) {
        let mut enabled = self.monitoring_enabled.write();
        if !*enabled {
            *enabled = true;
            self.notify.notify_one();
            log::debug!("System monitoring enabled");
        }
    }

    pub fn disable_monitoring(&self) {
        let mut enabled = self.monitoring_enabled.write();
        if *enabled {
            *enabled = false;
            log::debug!("System monitoring disabled");
        }
    }

    async fn monitor_worker(
        ui_tx: futures::channel::mpsc::UnboundedSender<SystemStats>,
        monitoring_enabled: Arc<RwLock<bool>>,
        notify: Arc<tokio::sync::Notify>,
    ) {
        let mut last_net_rx = 0u64;
        let mut last_net_tx = 0u64;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        loop {
            // Wait for monitoring to be enabled
            loop {
                if *monitoring_enabled.read() {
                    break;
                }
                notify.notified().await;
            }

            let (net_rx, net_tx) = Self::read_network_bytes().await;

            let net_down = if last_net_rx > 0 {
                net_rx.saturating_sub(last_net_rx) / 2
            } else {
                0
            };

            let net_up = if last_net_tx > 0 {
                net_tx.saturating_sub(last_net_tx) / 2
            } else {
                0
            };

            total_rx = total_rx.saturating_add(net_down);
            total_tx = total_tx.saturating_add(net_up);

            last_net_rx = net_rx;
            last_net_tx = net_tx;

            let stats = SystemStats {
                cpu: Self::read_cpu().await,
                cpu_temp: Self::read_cpu_temp().await,
                ram: Self::read_ram().await,
                gpu: Self::read_gpu().await,
                gpu_temp: Self::read_gpu_temp().await,
                disks: Self::read_disks().await,
                net_up,
                net_down,
                net_total: total_rx + total_tx,
            };

            let _ = ui_tx.unbounded_send(stats);

            // Sleep or wait for disable
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                _ = notify.notified() => {
                    if !*monitoring_enabled.read() {
                        log::debug!("Pausing system monitoring");
                    }
                }
            }
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
        // Try NVIDIA first
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(usage) = text.trim().parse::<u8>() {
                    return usage;
                }
            }
        }

        // Try AMD/Intel DRM sysfs
        for card in 0..4 {
            if let Ok(usage_str) = tokio::fs::read_to_string(format!(
                "/sys/class/drm/card{card}/device/gpu_busy_percent"
            ))
            .await
            {
                if let Ok(usage) = usage_str.trim().parse::<u8>() {
                    return usage;
                }
            }

            if let Ok(usage_str) = tokio::fs::read_to_string(format!(
                "/sys/class/drm/card{card}/gt/gt0/rps_cur_freq_mhz"
            ))
            .await
            {
                if let Ok(cur_freq) = usage_str.trim().parse::<u32>() {
                    if let Ok(max_str) = tokio::fs::read_to_string(format!(
                        "/sys/class/drm/card{card}/gt/gt0/rps_max_freq_mhz"
                    ))
                    .await
                    {
                        if let Ok(max_freq) = max_str.trim().parse::<u32>() {
                            return ((cur_freq * 100) / max_freq.max(1)) as u8;
                        }
                    }
                }
            }
        }

        0
    }

    async fn read_cpu_temp() -> Option<f32> {
        tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .await
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map(|t| t / 1000.0)
    }

    async fn read_gpu_temp() -> Option<f32> {
        // Try NVIDIA first
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(temp) = text.trim().parse::<f32>() {
                    return Some(temp);
                }
            }
        }

        // Try AMD/Intel DRM hwmon
        for card in 0..4 {
            for hwmon in 0..4 {
                if let Ok(temp_str) = tokio::fs::read_to_string(format!(
                    "/sys/class/drm/card{card}/device/hwmon/hwmon{hwmon}/temp1_input"
                ))
                .await
                {
                    if let Ok(temp) = temp_str.trim().parse::<f32>() {
                        return Some(temp / 1000.0);
                    }
                }
            }
        }

        None
    }

    async fn read_network_bytes() -> (u64, u64) {
        tokio::fs::read_to_string("/proc/net/dev")
            .await
            .ok()
            .map(|s| {
                let mut total_rx = 0u64;
                let mut total_tx = 0u64;

                for line in s.lines().skip(2) {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() >= 10 {
                        let iface = parts[0].trim_end_matches(':');
                        if iface != "lo" {
                            if let (Ok(rx), Ok(tx)) =
                                (parts[1].parse::<u64>(), parts[9].parse::<u64>())
                            {
                                total_rx += rx;
                                total_tx += tx;
                            }
                        }
                    }
                }

                (total_rx, total_tx)
            })
            .unwrap_or((0, 0))
    }

    async fn read_disks() -> Vec<DiskInfo> {
        if let Ok(output) = tokio::process::Command::new("df")
            .args(["-h", "--output=source,target,pcent"])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                return text
                    .lines()
                    .skip(1)
                    .filter_map(|line| {
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let source = parts[0];
                            let mount = parts[1];
                            let percent_str = parts[2].trim_end_matches('%');

                            if source.starts_with("/dev/")
                                && !source.contains("loop")
                                && mount != "/boot"
                            {
                                if let Ok(percent) = percent_str.parse::<u8>() {
                                    let name = source.strip_prefix("/dev/").unwrap_or(source);
                                    return Some(DiskInfo {
                                        name: name.to_string().into(),
                                        mount: mount.to_string().into(),
                                        percent,
                                    });
                                }
                            }
                        }
                        None
                    })
                    .collect();
            }
        }
        vec![]
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
