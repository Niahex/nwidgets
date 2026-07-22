use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_mb: f32,
}

pub fn get_running_processes() -> Vec<ProcessInfo> {
    let output = Command::new("ps").args(["aux", "--no-headers"]).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().filter_map(parse_ps_line).collect()
        }
        Err(_) => Vec::new(),
    }
}

fn parse_ps_line(line: &str) -> Option<ProcessInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 11 {
        return None;
    }

    let pid = parts.get(1)?.parse::<u32>().ok()?;
    let cpu_usage = parts.get(2)?.parse::<f32>().unwrap_or(0.0);
    let memory_mb = parts.get(5)?.parse::<f32>().unwrap_or(0.0) / 1024.0;
    let command = parts.get(10..)?.join(" ");
    let name = parts
        .get(10)?
        .split('/')
        .next_back()
        .unwrap_or(parts[10])
        .to_string();

    Some(ProcessInfo { pid, name, command, cpu_usage, memory_mb })
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    let output = Command::new("kill").arg("-9").arg(pid.to_string()).output();
    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => Err(format!("Failed to kill process {pid}")),
        Err(e) => Err(format!("Error killing process: {e}")),
    }
}

pub fn search_processes(query: &str) -> Vec<ProcessInfo> {
    let processes = get_running_processes();

    // "ps" seul → tous les processus
    if query == "ps" {
        return processes;
    }

    // "ps <term>" → filtre par nom
    let term = query.strip_prefix("ps").unwrap_or("").trim().to_lowercase();
    if term.is_empty() {
        return processes;
    }

    processes
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&term))
        .collect()
}
