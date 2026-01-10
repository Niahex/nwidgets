use gpui::{App, AsyncApp};
use std::sync::mpsc;
use zbus::{connection::Builder, interface};

pub struct DbusService;

#[derive(Debug)]
pub enum DbusCommand {
    ToggleChat,
    PinChat,
}

struct NWidgets {
    tx: mpsc::Sender<DbusCommand>,
}

#[interface(name = "org.nwidgets.App")]
impl NWidgets {
    async fn toggle_chat(&self) {
        let _ = self.tx.send(DbusCommand::ToggleChat);
    }

    async fn pin_chat(&self) {
        let _ = self.tx.send(DbusCommand::PinChat);
    }
}

impl DbusService {
    pub fn init(cx: &mut App) {
        let (tx, rx) = mpsc::channel::<DbusCommand>();

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
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(50))
                        .await;
                    while let Ok(cmd) = rx.try_recv() {
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
                        }
                    }
                }
            }
        })
        .detach();
    }
}
