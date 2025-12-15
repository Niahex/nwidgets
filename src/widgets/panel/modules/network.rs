use gpui::prelude::*;
use gpui::*;
use crate::services::network::{ConnectionType, NetworkService, NetworkStateChanged};

pub struct NetworkModule {
    network: Entity<NetworkService>,
}

impl NetworkModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let network = NetworkService::global(cx);

        cx.subscribe(&network, |_this, _network, _event: &NetworkStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { network }
    }
}

impl Render for NetworkModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.network.read(cx).state();

        let (icon, text) = match state.connection_type {
            ConnectionType::Ethernet => ("🌐", "Ethernet".to_string()),
            ConnectionType::Wifi { ssid, strength } => {
                let icon = if strength > 75 {
                    "📶"
                } else if strength > 50 {
                    "📶"
                } else if strength > 25 {
                    "📶"
                } else {
                    "📶"
                };
                (icon, ssid)
            }
            ConnectionType::Vpn { name } => ("🔒", name),
            ConnectionType::Disconnected => ("❌", "Disconnected".to_string()),
        };

        div()
            .flex()
            .gap_1()
            .items_center()
            .child(icon)
            .when(state.connected, |this| {
                this.child(
                    div()
                        .text_xs()
                        .max_w(px(100.))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(text)
                )
            })
    }
}
