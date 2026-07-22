use gpui::*;
use gpui_component::init as init_components;
use gpui_platform::application;

mod views;

fn main() {
    application().run(|cx: &mut App| {
        init_components(cx);
        gpui_tokio::init(cx);

        // ── Bar (panel) ──
        nwidgets_bar::open(cx, |_window, cx| cx.new(|_| views::bar::Bar))
            .expect("Failed to open bar");

        // ── Chat ──
        nwidgets_chat::open(cx, |_window, cx| cx.new(|_| views::chat::Chat))
            .expect("Failed to open chat");

        // ── OSD ──
        let osd = views::osd::OsdManager::new();
        {
            let mut mgr = osd.0.borrow_mut();
            mgr.open(cx, |_window, cx| cx.new(|_| views::osd::OsdView));
        }
        cx.set_global(osd);

        // ── Notifications ──
        let ntf = views::notification::NtfManager::new();
        {
            let mut mgr = ntf.0.borrow_mut();
            mgr.open(cx, |_window, cx| cx.new(|_| views::notification::NtfView));
        }
        cx.set_global(ntf);

        // ── Control Center ──
        nwidgets_control_center::open(cx, |_window, cx| {
            cx.new(|_| views::control_center::ControlCenter)
        })
        .expect("Failed to open control center");

        // ── Launcher ──
        nwidgets_launcher::open(cx, |_window, cx| cx.new(|_| views::launcher::Launcher))
            .expect("Failed to open launcher");

        // ── Shortcut (IPC / D-Bus Service) ──
        nwidgets_shortcut::ShortcutService::init(cx, |cmd, _cx| {
            println!("Received shortcut command: {:?}", cmd);
        });

        cx.activate(true);
    });
}
