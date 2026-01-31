use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub cpu: f32,
    pub mem: f32,
}

pub struct ProcessService;

impl ProcessService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if let Ok(pid) = name_str.parse::<u32>() {
                        if let Some(info) = self.get_process_info(pid) {
                            processes.push(info);
                        }
                    }
                }
            }
        }

        processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        processes
    }

    fn get_process_info(&self, pid: u32) -> Option<ProcessInfo> {
        let comm_path = format!("/proc/{}/comm", pid);
        let name = std::fs::read_to_string(&comm_path).ok()?.trim().to_string();

        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let cmd = std::fs::read_to_string(&cmdline_path)
            .ok()
            .map(|s| s.replace('\0', " ").trim().to_string())
            .unwrap_or_default();

        Some(ProcessInfo {
            pid,
            name,
            cmd,
            cpu: 0.0,
            mem: 0.0,
        })
    }

    pub fn kill_process(&self, pid: u32) -> anyhow::Result<()> {
        unsafe {
            if libc::kill(pid as i32, libc::SIGTERM) != 0 {
                anyhow::bail!("Failed to kill process {}", pid);
            }
        }
        Ok(())
    }
}
