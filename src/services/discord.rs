use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

pub struct DiscordToggled;

pub struct DiscordService {
    pub visible: bool,
}

impl EventEmitter<DiscordToggled> for DiscordService {}

struct GlobalDiscordService(Entity<DiscordService>);
impl Global for GlobalDiscordService {}

impl DiscordService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { visible: false }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalDiscordService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalDiscordService(service.clone()));
        service
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;
        cx.emit(DiscordToggled);
        cx.notify();
    }
}
