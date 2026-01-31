use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio::time::interval;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub ram_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub cpu_temp: f32,
    pub gpu_temp: f32,
}

#[derive(Clone)]
pub struct SystemMonitorService {
    state: Arc<RwLock<SystemStats>>,
}

impl SystemMonitorService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(SystemStats::default())),
        };

        service.start();
        service
    }

    fn start(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            let mut ticker = interval(Duration::from_secs(2));

            loop {
                ticker.tick().await;
                Self::update_stats(&state).await;
            }
        });
    }

    async fn update_stats(state: &Arc<RwLock<SystemStats>>) {
        let cpu = Self::read_cpu_usage().await.unwrap_or(0.0);
        let ram = Self::read_ram_usage().await.unwrap_or((0.0, 0.0, 0.0));
        let temps = Self::read_temperatures().await.unwrap_or((0.0, 0.0));
        let gpu = Self::read_gpu_usage().await.unwrap_or(0.0);

        let mut s = state.write();
        s.cpu_usage = cpu;
        s.ram_usage = ram.0;
        s.ram_used_gb = ram.1;
        s.ram_total_gb = ram.2;
        s.cpu_temp = temps.0;
        s.gpu_temp = temps.1;
        s.gpu_usage = gpu;
    }

    async fn read_cpu_usage() -> anyhow::Result<f32> {
        let content = tokio::fs::read_to_string("/proc/stat").await?;
        let line = content.lines().next().ok_or(anyhow::anyhow!("No CPU line"))?;
        let parts: Vec<u64> = line.split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() < 4 {
            return Ok(0.0);
        }

        let idle = parts[3];
        let total: u64 = parts.iter().sum();

        Ok(((total - idle) as f32 / total as f32) * 100.0)
    }

    async fn read_ram_usage() -> anyhow::Result<(f32, f32, f32)> {
        let content = tokio::fs::read_to_string("/proc/meminfo").await?;
        let mut total = 0u64;
        let mut available = 0u64;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        let used = total - available;
        let usage_pct = (used as f32 / total as f32) * 100.0;
        let used_gb = used as f32 / 1024.0 / 1024.0;
        let total_gb = total as f32 / 1024.0 / 1024.0;

        Ok((usage_pct, used_gb, total_gb))
    }

    async fn read_temperatures() -> anyhow::Result<(f32, f32)> {
        let mut cpu_temp = 0.0f32;
        let mut gpu_temp = 0.0f32;

        let hwmon_path = std::path::Path::new("/sys/class/hwmon");
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    let name = name.trim();

                    if name.contains("coretemp") || name.contains("k10temp") {
                        let temp_path = entry.path().join("temp1_input");
                        if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                            if let Ok(temp) = temp_str.trim().parse::<f32>() {
                                cpu_temp = temp / 1000.0;
                            }
                        }
                    }

                    if name.contains("amdgpu") || name.contains("nvidia") {
                        let temp_path = entry.path().join("temp1_input");
                        if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                            if let Ok(temp) = temp_str.trim().parse::<f32>() {
                                gpu_temp = temp / 1000.0;
                            }
                        }
                    }
                }
            }
        }

        Ok((cpu_temp, gpu_temp))
    }

    async fn read_gpu_usage() -> anyhow::Result<f32> {
        if let Ok(content) = tokio::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent").await {
            if let Ok(usage) = content.trim().parse::<f32>() {
                return Ok(usage);
            }
        }

        Ok(0.0)
    }

    pub fn get_stats(&self) -> SystemStats {
        self.state.read().clone()
    }
}
