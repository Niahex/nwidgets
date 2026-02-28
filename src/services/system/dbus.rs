use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AsyncApp};
use zbus::{connection::Builder, interface};

pub struct DbusService;

#[derive(Debug)]
pub enum DbusCommand {
    ToggleChat,
    PinChat,
    ToggleLauncher,
    ToggleJisig,
    ToggleDofusTools,
}
struct NWidgets {
    tx: mpsc::UnboundedSender<DbusCommand>,
}
#[interface(name = "org.nwidgets.App")]
impl NWidgets {
    async fn toggle_chat(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::ToggleChat);
    }
    async fn toggle_launcher(&self) {
        log::debug!("D-Bus toggle_launcher called");
        let _ = self.tx.unbounded_send(DbusCommand::ToggleLauncher);
    }
    async fn pin_chat(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::PinChat);
    }
    async fn toggle_jisig(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::ToggleJisig);
    }
    async fn toggle_dofustools(&self) {
        let _ = self.tx.unbounded_send(DbusCommand::ToggleDofusTools);
    }
}

impl DbusService {
    pub fn init(cx: &mut App) {
        let (tx, mut rx) = mpsc::unbounded::<DbusCommand>();

        // D-Bus server using gpui_tokio runtime
        gpui_tokio::Tokio::spawn(cx, async move {
            log::info!("Starting D-Bus server");
            match Builder::session() {
                Ok(builder) => {
                    let builder = match builder.name("org.nwidgets.App") {
                        Ok(b) => b,
                        Err(e) => {
                            log::error!("Failed to set D-Bus name: {e}");
                            return;
                        }
                    };

                    let builder = match builder.serve_at("/org/nwidgets/App", NWidgets { tx }) {
                        Ok(b) => b,
                        Err(e) => {
                            log::error!("Failed to serve D-Bus at path: {e}");
                            return;
                        }
                    };

                    match builder.build().await {
                        Ok(_conn) => {
                            log::info!("D-Bus service ready on org.nwidgets.App");
                            std::future::pending::<()>().await;
                        }
                        Err(e) => log::error!("Failed to build D-Bus connection: {e}"),
                    }
                }
                Err(e) => log::error!("Failed to create D-Bus builder: {e}"),
            }
        })
        .detach();

        // Command handler
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(cmd) = rx.next().await {
                    match cmd {
                        DbusCommand::ToggleChat => {
                            cx.update(|cx| {
                                let chat = crate::widgets::chat::ChatService::global(cx);
                                chat.update(cx, |chat, mcx| chat.toggle(mcx));
                            });
                        }
                        DbusCommand::PinChat => {
                            cx.update(|cx| {
                                let chat = crate::widgets::chat::ChatService::global(cx);
                                chat.update(cx, |chat, mcx| chat.toggle_pin(mcx));
                            });
                        }
                        DbusCommand::ToggleLauncher => {
                            log::debug!("Received ToggleLauncher command");
                            cx.update(|cx| {
                                let launcher = crate::widgets::launcher::LauncherService::global(cx);
                                launcher.update(cx, |launcher, mcx| {
                                    log::debug!(
                                        "Toggling launcher, current visible: {}",
                                        launcher.visible
                                    );
                                    launcher.toggle(mcx);
                                    log::debug!("After toggle, visible: {}", launcher.visible);
                                });
                            });
                        }
                        DbusCommand::ToggleJisig => {
                            cx.update(|cx| {
                                let jisig = crate::widgets::jisig::JisigService::global(cx);
                                jisig.update(cx, |jisig, mcx| jisig.toggle(mcx));
                            });
                        }
                        DbusCommand::ToggleDofusTools => {
                            cx.update(|cx| {
                                let dofustools = crate::widgets::dofustools::DofusToolsService::global(cx);
                                dofustools.update(cx, |dofustools, mcx| dofustools.toggle(mcx));
                            });
                        }
                    }
                }
            }
        })
        .detach();
    }
}
