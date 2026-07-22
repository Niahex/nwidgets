use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AsyncApp};
use zbus::{connection::Builder, interface};

#[derive(Debug, Clone)]
pub enum ShortcutCommand {
    ToggleChat,
    ToggleControlCenter,
    ToggleLauncher,
    PinChat,
}

struct NWidgetsShortcut {
    tx: mpsc::UnboundedSender<ShortcutCommand>,
}

#[interface(name = "org.nwidgets.App")]
impl NWidgetsShortcut {
    async fn toggle_chat(&self) {
        let _ = self.tx.unbounded_send(ShortcutCommand::ToggleChat);
    }

    async fn toggle_control_center(&self) {
        let _ = self.tx.unbounded_send(ShortcutCommand::ToggleControlCenter);
    }

    async fn toggle_launcher(&self) {
        let _ = self.tx.unbounded_send(ShortcutCommand::ToggleLauncher);
    }

    async fn pin_chat(&self) {
        let _ = self.tx.unbounded_send(ShortcutCommand::PinChat);
    }
}

pub struct ShortcutService;

impl ShortcutService {
    pub fn init<F>(cx: &mut App, mut handler: F)
    where
        F: FnMut(ShortcutCommand, &mut App) + 'static,
    {
        let (tx, mut rx) = mpsc::unbounded::<ShortcutCommand>();

        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(builder) = Builder::session() {
                if let Ok(builder) = builder.name("org.nwidgets.App") {
                    if let Ok(builder) = builder.serve_at("/org/nwidgets/App", NWidgetsShortcut { tx }) {
                        if let Ok(_conn) = builder.build().await {
                            std::future::pending::<()>().await;
                        }
                    }
                }
            }
        })
        .detach();

        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(cmd) = rx.next().await {
                    let _ = cx.update(|cx| {
                        handler(cmd, cx);
                    });
                }
            }
        })
        .detach();
    }
}
