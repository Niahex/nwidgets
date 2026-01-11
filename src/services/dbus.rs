use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AsyncApp};
use zbus::{connection::Builder, interface};

pub struct DbusService;

#[derive(Debug)]
pub enum DbusCommand {
    ToggleChat,
    PinChat,
    ToggleDiscord,
}

struct NWidgets {
    tx: mpsc::UnboundedSender<DbusCommand>,
}

#[interface(name = "org.nwidgets.App")]
impl NWidgets {
    async fn toggle_chat(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::ToggleChat);
    }

    async fn pin_chat(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::PinChat);
    }

    async fn toggle_discord(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::ToggleDiscord);
    }
}

impl DbusService {
    pub fn init(cx: &mut App) {
        let (tx, mut rx) = mpsc::unbounded::<DbusCommand>();

        // D-Bus server
        std::thread::spawn(move || {
            eprintln!("[DBUS] Starting D-Bus server thread");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match Builder::session() {
                    Ok(builder) => {
                        match builder
                            .name("org.nwidgets.App")
                            .unwrap()
                            .serve_at("/org/nwidgets/App", NWidgets { tx })
                            .unwrap()
                            .build()
                            .await
                        {
                            Ok(_conn) => {
                                eprintln!("[DBUS] ✅ Service ready on org.nwidgets.App");
                                std::future::pending::<()>().await;
                            }
                            Err(e) => eprintln!("[DBUS] ❌ Failed to build connection: {e}"),
                        }
                    }
                    Err(e) => eprintln!("[DBUS] ❌ Failed to create builder: {e}"),
                }
            });
        });

        // Command handler
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(cmd) = rx.next().await {
                    match cmd {
                        DbusCommand::ToggleChat => {
                            let _ = cx.update(|cx| {
                                let chat = super::chat::ChatService::global(cx);
                                chat.update(cx, |chat, mcx| chat.toggle(mcx));
                            });
                        }
                        DbusCommand::PinChat => {
                            let _ = cx.update(|cx| {
                                let chat = super::chat::ChatService::global(cx);
                                chat.update(cx, |chat, mcx| chat.toggle_pin(mcx));
                            });
                        }
                        DbusCommand::ToggleDiscord => {
                            let _ = cx.update(|cx| {
                                let discord = super::discord::DiscordService::global(cx);
                                discord.update(cx, |discord, mcx| discord.toggle(mcx));
                            });
                        }
                    }
                }
            }
        })
        .detach();
    }
}
