pub mod core;
pub mod fuzzy;
pub mod state;

use gpui::*;

pub use core::{LauncherCore, SearchResultType};

#[derive(Clone)]
pub struct LauncherToggled;

pub struct LauncherService {
    pub visible: bool,
}

impl EventEmitter<LauncherToggled> for LauncherService {}

impl LauncherService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;
        cx.emit(LauncherToggled);
        cx.notify();
    }
}

// Global accessor
struct GlobalLauncherService(Entity<LauncherService>);
impl Global for GlobalLauncherService {}

impl LauncherService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalLauncherService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalLauncherService(service.clone()));
        service
    }
}
