use gpui::{
    point, prelude::*, px, Application, Bounds, Size,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};

use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

mod modules;
mod services;
mod theme;
mod widgets;

use widgets::osd;
use widgets::Panel;

fn main() {
    Application::new().run(|cx| {
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(3440.), px(48.)),
                })),
                app_id: Some("nwidgets-panel".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-panel".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(px(48.)),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Panel::new),
        )
        .unwrap();

        // OSD - centered at bottom
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(1520.), px(980.)), // Center horizontally (3440/2 - 400/2 = 1520)
                    size: Size::new(px(400.), px(64.)),
                })),
                app_id: Some("nwidgets-osd".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-osd".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::BOTTOM,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| osd::Osd::new(osd::OsdType::CapsLock(false), cx)),
        )
        .unwrap();

        // Démarrer le service de notifications avec son manager
        use crate::services::notifications::NotificationManager;
        let _notification_manager = NotificationManager::new(cx);
        println!("[MAIN] 📢 Notification manager started");

        cx.activate(true);
    });
}
