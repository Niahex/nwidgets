use gpui::prelude::*;
use gpui::*;
use crate::services::network::{ConnectionType, NetworkService, NetworkStateChanged};
use crate::utils::Icon;

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
        
        Icon::new(state.get_icon_name())
            .size(px(16.))
            .preserve_colors(true)
    }
}
