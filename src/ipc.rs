use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

const SOCKET_PATH: &str = "/tmp/nwidgets.sock";

#[derive(Debug)]
pub enum IpcCommand {
    ToggleDictation,
}

impl IpcCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "dictation" => Some(IpcCommand::ToggleDictation),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            IpcCommand::ToggleDictation => "dictation".to_string(),
        }
    }
}

/// Start IPC server (used by main process)
pub fn start_ipc_server<F>(mut handler: F) -> std::io::Result<()>
where
    F: FnMut(IpcCommand) + Send + 'static,
{
    // Remove old socket if exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[IPC] Listening on {}", SOCKET_PATH);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0u8; 1024];
                    if let Ok(n) = stream.read(&mut buffer) {
                        let message = String::from_utf8_lossy(&buffer[..n]);
                        if let Some(cmd) = IpcCommand::from_str(message.trim()) {
                            println!("[IPC] Received command: {:?}", cmd);
                            handler(cmd);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[IPC] Connection error: {}", e);
                }
            }
        }
    });

    Ok(())
}

/// Send command to running instance (used by CLI)
pub fn send_command(cmd: IpcCommand) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(cmd.to_string().as_bytes())?;
    Ok(())
}
