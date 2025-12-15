mod services;
mod widgets;
mod utils;

use gpui::prelude::*;
use gpui::*;
use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions};
use services::{
    audio::AudioService,
    bluetooth::BluetoothService,
    hyprland::HyprlandService,
    mpris::MprisService,
    network::NetworkService,
    notifications::NotificationService,
    pomodoro::PomodoroService,
    systray::SystrayService,
};
use widgets::panel::Panel;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Initialize global services
        HyprlandService::init(cx);
        AudioService::init(cx);
        BluetoothService::init(cx);
        NetworkService::init(cx);
        MprisService::init(cx);
        PomodoroService::init(cx);
        SystrayService::init(cx);
        NotificationService::init(cx);

        // Create panel window with LayerShell
        cx.open_window(
            WindowOptions {
                window_bounds: None,
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-panel".to_string(),
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

        cx.activate(true);
    });
}
