mod services;
mod widgets;
mod utils;

use gpui::prelude::*;
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions, Layer};
use gpui::{Bounds, Point, Size, WindowBounds};
use services::{
    audio::AudioService,
    bluetooth::BluetoothService,
    hyprland::HyprlandService,
    mpris::MprisService,
    network::NetworkService,
    notifications::NotificationService,
    osd::{OsdService, OsdStateChanged},
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use widgets::{
    panel::Panel,
    osd::OsdWindowManager,
    notifications::NotificationsWindowManager,
};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use anyhow::Result;

struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        std::fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

fn main() {
    // Determine assets path - in development it's relative to the project root
    let assets_path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        // In production, assets should be alongside the binary
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    };

    Application::new()
        .with_assets(Assets { base: assets_path })
        .run(|cx: &mut App| {
        // Initialize global services
        HyprlandService::init(cx);
        AudioService::init(cx);
        BluetoothService::init(cx);
        NetworkService::init(cx);
        MprisService::init(cx);
        PomodoroService::init(cx);
        SystrayService::init(cx);
        NotificationService::init(cx);
        let osd_service = OsdService::init(cx);

        // Create panel window with LayerShell - full width (3440px), 50px height
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(0.0),
                        y: px(0.0),
                    },
                    size: Size {
                        width: px(3440.0),
                        height: px(50.0),
                    },
                })),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-panel".to_string(),
                    layer : Layer::Top,
                    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(px(50.)),
                    margin: None,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| Panel::new(cx)),
        )
        .unwrap();

        // Gestionnaire de fenêtre OSD
        let osd_manager = Arc::new(Mutex::new(OsdWindowManager::new()));
        let osd_manager_clone = Arc::clone(&osd_manager);

        // S'abonner aux changements d'état OSD pour ouvrir/fermer la fenêtre
        cx.subscribe(&osd_service, move |_osd, event: &OsdStateChanged, cx| {
            let mut manager = osd_manager_clone.lock();
            
            if event.visible {
                println!("[MAIN] 🪟 Opening OSD window");
                manager.open_window(cx);
            } else {
                println!("[MAIN] 🚪 Closing OSD window");
                manager.close_window(cx);
            }
        })
        .detach();

        // TODO: Gestionnaire de fenêtre Notifications
        // Pour l'instant, on garde l'ancienne méthode pour les notifications
        // Tu pourras l'adapter de la même manière que l'OSD
        
        cx.activate(true);
    });
}
