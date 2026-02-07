use crate::widgets::launcher::types::ProcessInfo;
use std::process::Command;

pub fn get_running_processes() -> Vec<ProcessInfo> {
    let output = Command::new("ps").args(["aux", "--no-headers"]).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().filter_map(parse_ps_line).collect()
        }
        Err(_) => Vec::with_capacity(0),
    }
}

fn parse_ps_line(line: &str) -> Option<ProcessInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 11 {
        return None;
    }

    let pid = parts[1].parse::<u32>().ok()?;
    let cpu_usage = parts[2].parse::<f32>().unwrap_or(0.0);
    let memory_mb = parts[5].parse::<f32>().unwrap_or(0.0) / 1024.0; // KB to MB
    let command = parts[10..].join(" ");
    let name = parts[10]
        .split('/')
        .next_back()
        .unwrap_or(parts[10])
        .to_string();

    Some(ProcessInfo {
        pid,
        name,
        command,
        cpu_usage,
        memory_mb,
    })
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    let output = Command::new("kill").arg("-9").arg(pid.to_string()).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(format!("Failed to kill process {pid}"))
            }
        }
        Err(e) => Err(format!("Error killing process: {e}")),
    }
}

pub fn is_process_query(query: &str) -> bool {
    query.starts_with("kill ") || query.starts_with("ps")
}
