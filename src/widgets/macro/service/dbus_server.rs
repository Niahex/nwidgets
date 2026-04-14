use crate::widgets::r#macro::types::Macro;
use futures::channel::mpsc::UnboundedSender;
use zbus::Connection;

#[derive(Debug)]
pub enum MacroDbusCommand {
    Toggle,
    StartRecording(String),
    StopRecording,
    PlayMacro(uuid::Uuid),
    StopPlayback,
}

pub struct MacroDbusServer {
    command_tx: UnboundedSender<MacroDbusCommand>,
    macros: std::sync::Arc<parking_lot::RwLock<Vec<Macro>>>,
}

impl MacroDbusServer {
    pub fn new(
        command_tx: UnboundedSender<MacroDbusCommand>,
        macros: std::sync::Arc<parking_lot::RwLock<Vec<Macro>>>,
    ) -> Self {
        Self { command_tx, macros }
    }
}

#[zbus::interface(name = "org.nwidgets.Macro")]
impl MacroDbusServer {
    fn toggle(&self) {
        log::info!("D-Bus: Toggle macro window");
        if let Err(e) = self.command_tx.unbounded_send(MacroDbusCommand::Toggle) {
            log::warn!("Failed to send D-Bus toggle macro command: {}", e);
        }
    }

    fn start_recording(&self, name: String) {
        log::info!("D-Bus: Start recording macro: {}", name);
        if let Err(e) = self.command_tx.unbounded_send(MacroDbusCommand::StartRecording(name)) {
            log::warn!("Failed to send D-Bus start recording command: {}", e);
        }
    }

    fn stop_recording(&self) {
        log::info!("D-Bus: Stop recording");
        if let Err(e) = self.command_tx.unbounded_send(MacroDbusCommand::StopRecording) {
            log::warn!("Failed to send D-Bus stop recording command: {}", e);
        }
    }

    fn play_macro(&self, macro_id: String) {
        log::info!("D-Bus: Play macro: {}", macro_id);
        if let Ok(uuid) = uuid::Uuid::parse_str(&macro_id) {
            if let Err(e) = self.command_tx.unbounded_send(MacroDbusCommand::PlayMacro(uuid)) {
                log::warn!("Failed to send D-Bus play macro command: {}", e);
            }
        }
    }

    fn stop_playback(&self) {
        log::info!("D-Bus: Stop playback");
        if let Err(e) = self.command_tx.unbounded_send(MacroDbusCommand::StopPlayback) {
            log::warn!("Failed to send D-Bus stop playback command: {}", e);
        }
    }

    fn list_macros(&self) -> Vec<String> {
        self.macros.read().iter().map(|m| format!("{}:{}", m.id, m.name)).collect()
    }
}

pub async fn run_dbus_server(
    command_tx: UnboundedSender<MacroDbusCommand>,
    macros: std::sync::Arc<parking_lot::RwLock<Vec<Macro>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::session().await?;
    let server = MacroDbusServer::new(command_tx, macros);

    connection
        .object_server()
        .at("/org/nwidgets/Macro", server)
        .await?;

    connection.request_name("org.nwidgets.Macro").await?;

    log::info!("Macro D-Bus service ready on org.nwidgets.Macro");
    std::future::pending::<()>().await;
    Ok(())
}
