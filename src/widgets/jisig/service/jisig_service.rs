use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

use crate::widgets::jisig::types::JisigToggled;

pub struct JisigService {
    pub visible: bool,
}

impl EventEmitter<JisigToggled> for JisigService {}

struct GlobalJisigService(Entity<JisigService>);
impl Global for GlobalJisigService {}

impl JisigService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
        }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalJisigService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalJisigService(service.clone()));
        service
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;

        if self.visible {
            let cc = crate::widgets::control_center::ControlCenterService::global(cx);
            cc.update(cx, |cc, cx| {
                if cc.is_visible() {
                    cc.toggle(cx);
                }
            });
        }

        cx.emit(JisigToggled);
        cx.notify();
    }
}
