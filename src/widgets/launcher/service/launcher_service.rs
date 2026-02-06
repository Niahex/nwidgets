use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

use crate::widgets::launcher::types::LauncherToggled;
use crate::widgets::launcher::core::applications;

pub struct LauncherService {
    pub visible: bool,
}

impl EventEmitter<LauncherToggled> for LauncherService {}

struct GlobalLauncherService(Entity<LauncherService>);
impl Global for GlobalLauncherService {}

impl LauncherService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { visible: false }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalLauncherService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalLauncherService(service.clone()));

        cx.spawn(|cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                cx.background_executor()
                    .spawn(async {
                        let apps = applications::scan_applications();
                        let _ = applications::save_to_cache(&apps);
                    })
                    .await;
            }
        })
        .detach();

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

        cx.emit(LauncherToggled);
        cx.notify();
    }
}
