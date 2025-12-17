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
    osd::OsdService,
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use widgets::{panel::Panel, osd::OsdWidget};
use std::path::PathBuf;
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
        OsdService::init(cx);

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

        // Create OSD window with LayerShell - centered at bottom
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px((3440.0 - 400.0) / 2.0), // Centré horizontalement
                        y: px(1440.0 - 64.0 - 80.0),    // 80px du bas
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
                    exclusive_zone: None,
                    margin: Some((px(0.0), px(0.0), px(80.0), px(0.0))), // 80px bottom margin
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| OsdWidget::new(cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
