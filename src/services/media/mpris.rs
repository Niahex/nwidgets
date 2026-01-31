use std::sync::Arc;
use parking_lot::RwLock;
use zbus::Connection;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct MprisPlayer {
    pub name: String,
    pub identity: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: Option<String>,
    pub is_playing: bool,
    pub position: i64,
    pub length: i64,
}

#[derive(Clone)]
pub struct MprisService {
    state: Arc<RwLock<MprisState>>,
}

#[derive(Default)]
struct MprisState {
    players: Vec<MprisPlayer>,
    active_player: Option<String>,
}

impl MprisService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(MprisState::default())),
        };

        service.start();
        service
    }

    fn start(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::monitor_players(state).await {
                log::error!("MPRIS monitor error: {}", e);
            }
        });
    }

    async fn monitor_players(_state: Arc<RwLock<MprisState>>) -> anyhow::Result<()> {
        let _connection = Connection::session().await?;

        log::info!("MPRIS service started");

        Ok(())
    }

    pub fn get_active_player(&self) -> Option<MprisPlayer> {
        let state = self.state.read();
        state.active_player.as_ref()
            .and_then(|name| state.players.iter().find(|p| &p.name == name))
            .cloned()
    }

    pub fn play(&self) {
        log::info!("MPRIS: play");
    }

    pub fn pause(&self) {
        log::info!("MPRIS: pause");
    }

    pub fn play_pause(&self) {
        log::info!("MPRIS: play_pause");
    }

    pub fn next(&self) {
        log::info!("MPRIS: next");
    }

    pub fn previous(&self) {
        log::info!("MPRIS: previous");
    }
}
