use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::mpsc;
use zbus::{proxy, Connection};

#[derive(Debug, Clone, Default)]
pub struct MprisMetadata {
    pub title: String,
    pub artist: String,
    pub art_url: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl From<String> for PlaybackStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MprisState {
    pub metadata: MprisMetadata,
    pub status: PlaybackStatus,
}

// MPRIS Interface
#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer2Player {
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, zbus::zvariant::OwnedValue>>;
    #[zbus(property)]
    fn volume(&self) -> zbus::Result<f64>;
    #[zbus(property)]
    fn set_volume(&self, volume: f64) -> zbus::Result<()>;

    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn play_pause(&self) -> zbus::Result<()>;
}

pub struct MprisService;

// Constantes pour Spotify
const SPOTIFY_DEST: &str = "org.mpris.MediaPlayer2.spotify";
const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";

impl MprisService {
    pub fn subscribe<F>(callback: F)
    where
        F: Fn(MprisState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            crate::utils::runtime::block_on(async {
                let connection = match Connection::session().await {
                    Ok(c) => c,
                    Err(_) => return,
                };

                // État initial vide
                let mut last_state = MprisState::default();
                let _ = tx.send(last_state.clone());

                loop {
                    // Essayer de se connecter à Spotify
                    let proxy_result = MediaPlayer2PlayerProxy::builder(&connection)
                        .destination(SPOTIFY_DEST)
                        .and_then(|b| b.path(MPRIS_PATH))
                        .map(|b| b.build());

                    if let Ok(builder_future) = proxy_result {
                        match builder_future.await {
                            Ok(proxy) => {
                                // 1. Récupérer l'état initial actuel
                                if let Ok(state) = Self::get_spotify_state(&proxy).await {
                                    if tx.send(state.clone()).is_err() {
                                        break;
                                    }
                                    last_state = state;
                                }

                                // 2. Écouter les changements (Signal PropertiesChanged)
                                // On écoute les changements de playback_status pour détecter play/pause
                                let mut status_stream =
                                    proxy.receive_playback_status_changed().await;

                                // On écoute aussi les changements de metadata pour les changements de piste
                                let mut metadata_stream = proxy.receive_metadata_changed().await;

                                // On écoute les changements de volume
                                let mut volume_stream = proxy.receive_volume_changed().await;

                                loop {
                                    tokio::select! {
                                        status_change = status_stream.next() => {
                                            if status_change.is_none() { break; }
                                            if let Ok(state) = Self::get_spotify_state(&proxy).await {
                                                if tx.send(state.clone()).is_err() {
                                                    return;
                                                }
                                                last_state = state;
                                            }
                                        }
                                        metadata_change = metadata_stream.next() => {
                                            if metadata_change.is_none() { break; }
                                            if let Ok(state) = Self::get_spotify_state(&proxy).await {
                                                if tx.send(state.clone()).is_err() {
                                                    return;
                                                }
                                                last_state = state;
                                            }
                                        }
                                        volume_change = volume_stream.next() => {
                                            if volume_change.is_none() { break; }
                                            if let Ok(state) = Self::get_spotify_state(&proxy).await {
                                                if tx.send(state.clone()).is_err() {
                                                    return;
                                                }
                                                last_state = state;
                                            }
                                        }
                                    }
                                }
                                // Si on sort de la boucle while, c'est que Spotify a probablement fermé ou le stream a coupé
                            }
                            Err(_) => {
                                // Spotify n'est pas ouvert ou erreur de connexion au proxy
                                // On envoie un état "Stopped" si on n'y est pas déjà
                                if last_state.status != PlaybackStatus::Stopped {
                                    last_state = MprisState::default();
                                    let _ = tx.send(last_state.clone());
                                }
                            }
                        }
                    }

                    // Si on est ici, c'est que la connexion a échoué ou a été perdue.
                    // On attend un peu avant de réessayer de détecter Spotify.
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            });
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    async fn get_spotify_state(proxy: &MediaPlayer2PlayerProxy<'_>) -> zbus::Result<MprisState> {
        let status_str = proxy
            .playback_status()
            .await
            .unwrap_or_else(|_| "Stopped".to_string());
        let status = PlaybackStatus::from(status_str);

        let mut metadata = MprisMetadata::default();

        if let Ok(metadata_map) = proxy.metadata().await {
            if let Some(v) = metadata_map.get("xesam:title") {
                metadata.title = v
                    .downcast_ref::<zbus::zvariant::Str>()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
            }
            if let Some(v) = metadata_map.get("xesam:artist") {
                if let Ok(arr) = v.downcast_ref::<zbus::zvariant::Array>() {
                    if let Ok(Some(first)) = arr.get::<zbus::zvariant::Value>(0) {
                        metadata.artist = first
                            .downcast_ref::<zbus::zvariant::Str>()
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                    }
                } else if let Ok(s) = v.downcast_ref::<zbus::zvariant::Str>() {
                    metadata.artist = s.to_string();
                }
            }
            if let Some(v) = metadata_map.get("mpris:artUrl") {
                metadata.art_url = v
                    .downcast_ref::<zbus::zvariant::Str>()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
            }
        }

        Ok(MprisState { metadata, status })
    }

    // Commandes pour Spotify uniquement

    pub fn play_pause() {
        Self::run_spotify_command(|p| async move { p.play_pause().await });
    }

    pub fn next() {
        Self::run_spotify_command(|p| async move { p.next().await });
    }

    pub fn previous() {
        Self::run_spotify_command(|p| async move { p.previous().await });
    }

    pub fn volume_up() {
        Self::run_spotify_command(|p| async move {
            if let Ok(vol) = p.volume().await {
                p.set_volume((vol + 0.05).min(1.0)).await
            } else {
                Ok(())
            }
        });
    }

    pub fn volume_down() {
        Self::run_spotify_command(|p| async move {
            if let Ok(vol) = p.volume().await {
                p.set_volume((vol - 0.05).max(0.0)).await
            } else {
                Ok(())
            }
        });
    }

    fn run_spotify_command<F, Fut>(action: F)
    where
        F: FnOnce(MediaPlayer2PlayerProxy<'static>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = zbus::Result<()>> + Send,
    {
        std::thread::spawn(move || {
            crate::utils::runtime::block_on(async {
                if let Ok(conn) = Connection::session().await {
                    let proxy_builder = MediaPlayer2PlayerProxy::builder(&conn)
                        .destination(SPOTIFY_DEST)
                        .and_then(|b| b.path(MPRIS_PATH))
                        .map(|b| b.build());

                    if let Ok(future) = proxy_builder {
                        if let Ok(proxy) = future.await {
                            let _ = action(proxy).await;
                        }
                    }
                }
            });
        });
    }
}
