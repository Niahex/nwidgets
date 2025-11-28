use std::collections::HashMap;
use std::sync::mpsc;
use zbus::{proxy, Connection};

#[derive(Debug, Clone, Default)]
pub struct MprisMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        PlaybackStatus::Stopped
    }
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
    pub player_name: String,
}

// MPRIS MediaPlayer2.Player interface
#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2"
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

impl MprisService {
    pub fn subscribe<F>(callback: F)
    where
        F: Fn(MprisState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Thread qui monitore MPRIS
        std::thread::spawn(move || {
            crate::utils::runtime::block_on(async {
                let mut last_state = MprisState::default();

                // Envoyer l'état initial
                let _ = tx.send(last_state.clone());

                loop {
                    if let Ok(state) = Self::get_mpris_state().await {
                        // Seulement envoyer si l'état a changé
                        if state.status != last_state.status
                            || state.metadata.title != last_state.metadata.title
                            || state.metadata.artist != last_state.metadata.artist
                        {
                            if tx.send(state.clone()).is_err() {
                                break;
                            }
                            last_state = state;
                        }
                    }

                    // Poll toutes les 2 secondes
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    async fn get_mpris_state() -> zbus::Result<MprisState> {
        let connection = Connection::session().await?;

        // Trouver tous les lecteurs MPRIS
        let proxy = zbus::fdo::DBusProxy::new(&connection).await?;
        let names = proxy.list_names().await?;

        // Chercher UNIQUEMENT Spotify
        for name in names {
            if name.as_str() == "org.mpris.MediaPlayer2.spotify" {
                if let Ok(state) = Self::get_player_state(&connection, &name).await {
                    if state.status != PlaybackStatus::Stopped
                        && !state.metadata.title.is_empty()
                    {
                        return Ok(state);
                    }
                }
            }
        }

        Ok(MprisState::default())
    }

    async fn get_player_state(
        connection: &Connection,
        player_name: &str,
    ) -> zbus::Result<MprisState> {
        let player_proxy = MediaPlayer2PlayerProxy::builder(connection)
            .destination(player_name)?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        let status_str = player_proxy.playback_status().await?;
        let status = PlaybackStatus::from(status_str);

        let metadata_map = player_proxy.metadata().await?;
        let metadata = Self::parse_metadata(metadata_map);

        // Extraire le nom du lecteur (ex: "org.mpris.MediaPlayer2.spotify" -> "spotify")
        let player_name = player_name
            .strip_prefix("org.mpris.MediaPlayer2.")
            .unwrap_or(player_name)
            .to_string();

        Ok(MprisState {
            metadata,
            status,
            player_name,
        })
    }

    fn parse_metadata(
        map: HashMap<String, zbus::zvariant::OwnedValue>,
    ) -> MprisMetadata {
        let mut metadata = MprisMetadata::default();

        // Titre
        if let Some(title) = map.get("xesam:title") {
            if let Ok(s) = title.downcast_ref::<zbus::zvariant::Str>() {
                metadata.title = s.to_string();
            }
        }

        // Artiste (peut être un tableau)
        if let Some(artist) = map.get("xesam:artist") {
            if let Ok(arr) = artist.downcast_ref::<zbus::zvariant::Array>() {
                if let Ok(Some(first)) = arr.get::<zbus::zvariant::Value>(0) {
                    if let Ok(s) = first.downcast_ref::<zbus::zvariant::Str>() {
                        metadata.artist = s.to_string();
                    }
                }
            }
        }

        // Album
        if let Some(album) = map.get("xesam:album") {
            if let Ok(s) = album.downcast_ref::<zbus::zvariant::Str>() {
                metadata.album = s.to_string();
            }
        }

        // Art URL
        if let Some(art_url) = map.get("mpris:artUrl") {
            if let Ok(s) = art_url.downcast_ref::<zbus::zvariant::Str>() {
                metadata.art_url = s.to_string();
            }
        }

        metadata
    }

    /// Passer à la piste suivante
    pub fn next() {
        std::thread::spawn(|| {
            crate::utils::runtime::block_on(async {
                if let Err(e) = Self::next_async().await {
                    eprintln!("Failed to skip to next track: {}", e);
                }
            });
        });
    }

    async fn next_async() -> zbus::Result<()> {
        let connection = Connection::session().await?;

        // Uniquement Spotify
        let player_proxy = MediaPlayer2PlayerProxy::builder(&connection)
            .destination("org.mpris.MediaPlayer2.spotify")?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        player_proxy.next().await
    }

    /// Revenir à la piste précédente
    pub fn previous() {
        std::thread::spawn(|| {
            crate::utils::runtime::block_on(async {
                if let Err(e) = Self::previous_async().await {
                    eprintln!("Failed to skip to previous track: {}", e);
                }
            });
        });
    }

    async fn previous_async() -> zbus::Result<()> {
        let connection = Connection::session().await?;

        // Uniquement Spotify
        let player_proxy = MediaPlayer2PlayerProxy::builder(&connection)
            .destination("org.mpris.MediaPlayer2.spotify")?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        player_proxy.previous().await
    }

    /// Toggle play/pause
    pub fn play_pause() {
        std::thread::spawn(|| {
            crate::utils::runtime::block_on(async {
                if let Err(e) = Self::play_pause_async().await {
                    eprintln!("Failed to toggle play/pause: {}", e);
                }
            });
        });
    }

    async fn play_pause_async() -> zbus::Result<()> {
        let connection = Connection::session().await?;

        // Uniquement Spotify
        let player_proxy = MediaPlayer2PlayerProxy::builder(&connection)
            .destination("org.mpris.MediaPlayer2.spotify")?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        player_proxy.play_pause().await
    }

    /// Augmenter le volume
    pub fn volume_up() {
        std::thread::spawn(|| {
            crate::utils::runtime::block_on(async {
                if let Err(e) = Self::adjust_volume_async(0.05).await {
                    eprintln!("Failed to increase volume: {}", e);
                }
            });
        });
    }

    /// Diminuer le volume
    pub fn volume_down() {
        std::thread::spawn(|| {
            crate::utils::runtime::block_on(async {
                if let Err(e) = Self::adjust_volume_async(-0.05).await {
                    eprintln!("Failed to decrease volume: {}", e);
                }
            });
        });
    }

    async fn adjust_volume_async(delta: f64) -> zbus::Result<()> {
        let connection = Connection::session().await?;

        // Uniquement Spotify
        let player_proxy = MediaPlayer2PlayerProxy::builder(&connection)
            .destination("org.mpris.MediaPlayer2.spotify")?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        let current_volume = player_proxy.volume().await?;
        let new_volume = (current_volume + delta).clamp(0.0, 1.0);
        player_proxy.set_volume(new_volume).await
    }
}
