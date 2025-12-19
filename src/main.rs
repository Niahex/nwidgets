mod services;
mod utils;
mod widgets;

use anyhow::Result;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::prelude::*;
use gpui::*;
use gpui::{Bounds, Point, Size, WindowBounds};
use parking_lot::Mutex;
use services::{
    audio::AudioService,
    bluetooth::BluetoothService,
    hyprland::HyprlandService,
    mpris::MprisService,
    network::NetworkService,
    notifications::{NotificationAdded, NotificationService},
    osd::{OsdService, OsdStateChanged},
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use std::path::PathBuf;
use std::sync::Arc;
use widgets::{
    notifications::{NotificationsStateChanged, NotificationsWindowManager},
    osd::OsdWidget,
    panel::Panel,
};

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
            let notif_service = NotificationService::init(cx);
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
                        layer: Layer::Top,
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

            // 1. Ouvrir la fenêtre OSD UNE SEULE FOIS au démarrage
            let osd_window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Point {
                                x: px((3440.0 - 400.0) / 2.0),
                                y: px(1440.0 - 64.0 - 80.0),
                            },
                            size: Size {
                                width: px(400.0),
                                height: px(64.0),
                            },
                        })),
                        titlebar: None,
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nwidgets-osd".to_string(),
                            layer: Layer::Overlay,
                            anchor: Anchor::BOTTOM,
                            exclusive_zone: None, // Important pour ne pas décaler les fenêtres
                            margin: Some((px(0.0), px(0.0), px(80.0), px(0.0))),
                            keyboard_interactivity: KeyboardInteractivity::None,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    |_window, cx| cx.new(|cx| OsdWidget::new(cx)),
                )
                .unwrap();

            // 2. La visibilité est gérée par l'opacité dans le widget
            cx.subscribe(&osd_service, move |_osd, _event: &OsdStateChanged, _cx| {
                // Le widget se met à jour automatiquement via son abonnement
            })
            .detach();

            // Gestionnaire de fenêtre notifications
            let notif_manager = Arc::new(Mutex::new(NotificationsWindowManager::new()));
            let notif_manager_clone = Arc::clone(&notif_manager);

            // Ouvrir la fenêtre à la première notification
            cx.subscribe(
                &notif_service,
                move |_service, event: &NotificationAdded, cx| {
                    let mut manager = notif_manager_clone.lock();

                    if let Some(widget) = manager.open_window(cx) {
                        let notif_manager_clone2 = Arc::clone(&notif_manager_clone);
                        cx.subscribe(
                            &widget,
                            move |_widget, event: &NotificationsStateChanged, cx| {
                                if !event.has_notifications {
                                    let mut manager = notif_manager_clone2.lock();
                                    manager.close_window(cx);
                                }
                            },
                        )
                        .detach();
                    }
                },
            )
            .detach();

            cx.activate(true);
        });
}
