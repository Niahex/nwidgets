use gpui::*;
use gpui_component::init as init_components;
use gpui_platform::application;

mod views;

fn main() {
    application().run(|cx: &mut App| {
        init_components(cx);
        gpui_tokio::init(cx);

        // ActiveTheme transparency configuration for Root windows
        let mut theme = gpui_component::Theme::global(cx).clone();
        let transparent = gpui::rgb(0x000000).opacity(0.0).into();
        theme.colors.background = transparent;
        theme.tokens.background = gpui_component::ThemeToken::from(transparent);
        cx.set_global(theme);

        // ── Services Initialization ──
        let _niri_service = nwidgets_service_niri::NiriActiveWindowService::init(cx);
        let _audio_service = nwidgets_service_audio::AudioService::init(cx);
        let _system_monitor_service = nwidgets_service_system_monitor::SystemMonitorService::init(cx);
        let _bluetooth_service = nwidgets_service_bluetooth::BluetoothService::init(cx);
        let _network_service = nwidgets_service_network::NetworkService::init(cx);
        let _applications_service = nwidgets_service_applications::ApplicationsService::init(cx);
        let _clipboard_service = nwidgets_service_clipboard::ClipboardService::init(cx);
        let _lock_service = nwidgets_service_lock::LockMonitor::init(cx);

        // ── Launcher Window ──
        cx.bind_keys([
            KeyBinding::new("escape", views::chat::CloseChat, None),
            KeyBinding::new("escape", views::launcher::CloseLauncher, None),
        ]);

        // ── Chat ──
        let mut chat_fh = None;
        let mut chat_entity = None;
        let chat_window = nwidgets_chat::open(cx, |window, cx| {
            let view = cx.new(views::chat::Chat::new);
            chat_fh = Some(view.read(cx).focus_handle.clone());
            chat_entity = Some(view.clone());
            cx.new(|cx| gpui_component::Root::new(view, window, cx).bordered(false))
        })
        .expect("Failed to open chat");
        let chat_visible = std::rc::Rc::new(std::cell::Cell::new(false));

        // Subscribe Escape event for Chat
        if let Some(chat_entity) = chat_entity {
            let chat_win_close = chat_window.clone();
            let chat_vis_close = chat_visible.clone();
            let _ = chat_window.update(cx, |_, _window, cx| {
                cx.subscribe(
                    &chat_entity,
                    move |_this, _emitter, _event: &views::chat::CloseChat, cx| {
                        chat_vis_close.set(false);
                        nwidgets_chat::set_visible(&chat_win_close, false, cx);
                    },
                )
                .detach();
            });
        }

        // ── OSD ──
        let mut osd_entity = None;
        let osd_window = nwidgets_osd::open(cx, |window, cx| {
            let view = cx.new(views::osd::OsdView::new);
            osd_entity = Some(view.clone());
            cx.new(|cx| gpui_component::Root::new(view, window, cx).bordered(false))
        });
        if let (Some(osd_window), Some(osd_entity)) = (osd_window, osd_entity) {
            let handle: AnyWindowHandle = osd_window.into();
            let _ = osd_window.update(cx, |_, _window, cx| {
                osd_entity.update(cx, |osd, _| {
                    osd.set_window_handle(handle);
                });
            });
        }

        // ── Notifications ──
        let mut ntf_entity = None;
        let ntf_window = nwidgets_notification::open(cx, |window, cx| {
            let view = cx.new(views::notification::NtfView::new);
            ntf_entity = Some(view.clone());
            cx.new(|cx| gpui_component::Root::new(view, window, cx).bordered(false))
        });
        if let (Some(ntf_window), Some(ntf_entity)) = (ntf_window, ntf_entity) {
            let handle: AnyWindowHandle = ntf_window.into();
            let _ = ntf_window.update(cx, |_, _window, cx| {
                ntf_entity.update(cx, |ntf, _| {
                    ntf.set_window_handle(handle);
                });
            });
        }

        // ── Control Center ──
        let cc_window = nwidgets_control_center::open(cx, |window, cx| {
            let view = cx.new(views::control_center::ControlCenter::new);
            cx.new(|cx| gpui_component::Root::new(view, window, cx).bordered(false))
        })
        .expect("Failed to open control center");
        let cc_visible = std::rc::Rc::new(std::cell::Cell::new(false));

        // ── Bar (panel) ──
        let cc_window_for_bar = cc_window.clone();
        nwidgets_bar::open(cx, move |window, cx| {
            let cc = cc_window_for_bar.into();
            let view = cx.new(move |cx| views::bar::Bar::new(cc, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx).bordered(false))
        })
        .expect("Failed to open bar");

        // ── Launcher ──
        let mut launcher_fh = None;
        let mut launcher_entity = None;
        let launcher_window = nwidgets_launcher::open(cx, |window, cx| {
            let launcher_view = cx.new(|cx| views::launcher::Launcher::new(window, cx));
            launcher_fh = Some(launcher_view.read(cx).focus_handle.clone());
            launcher_entity = Some(launcher_view.clone());
            cx.new(|cx| gpui_component::Root::new(launcher_view, window, cx).bordered(false))
        })
        .expect("Failed to open launcher");
        let launcher_visible = std::rc::Rc::new(std::cell::Cell::new(false));

        // Subscribe Escape event for Launcher
        if let Some(ref launcher_entity) = launcher_entity {
            let launcher_win_close = launcher_window.clone();
            let launcher_vis_close = launcher_visible.clone();
            let _ = launcher_window.update(cx, |_, _window, cx| {
                cx.subscribe(
                    &launcher_entity,
                    move |_this, _emitter, _event: &views::launcher::CloseLauncher, cx| {
                        launcher_vis_close.set(false);
                        nwidgets_launcher::set_visible(&launcher_win_close, false, None, cx);
                    },
                )
                .detach();
            });
        }

        // ── Shortcut (IPC / D-Bus Service) ──
        let chat_win = chat_window.clone();
        let chat_vis = chat_visible.clone();
        let cc_win = cc_window.clone();
        let cc_vis = cc_visible.clone();
        let launcher_win = launcher_window.clone();
        let launcher_vis = launcher_visible.clone();

        nwidgets_shortcut::ShortcutService::init(cx, move |cmd, cx| match cmd {
            nwidgets_shortcut::ShortcutCommand::ToggleChat => {
                let v = !chat_vis.get();
                chat_vis.set(v);
                nwidgets_chat::set_visible(&chat_win, v, cx);
                if v {
                    if let Some(ref fh) = chat_fh {
                        let fh = fh.clone();
                        let _ = chat_win.update(cx, |_, window, cx| {
                            window.focus(&fh, cx);
                        });
                    }
                }
            }
            nwidgets_shortcut::ShortcutCommand::ToggleControlCenter => {
                let v = !cc_vis.get();
                cc_vis.set(v);
                nwidgets_control_center::set_visible(&cc_win, v, cx);
            }
            nwidgets_shortcut::ShortcutCommand::ToggleLauncher => {
                let v = !launcher_vis.get();
                launcher_vis.set(v);
                if v {
                    if let Some(ref entity) = launcher_entity {
                        let _ = launcher_window.update(cx, |_, window, cx| {
                            entity.update(cx, |launcher, cx| {
                                launcher.reset(window, cx);
                            });
                        });
                    }
                }
                nwidgets_launcher::set_visible(&launcher_win, v, launcher_fh.as_ref(), cx);
            }
            nwidgets_shortcut::ShortcutCommand::PinChat => {}
        });

        cx.activate(true);
    });
}
