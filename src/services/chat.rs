use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

pub struct ChatToggled;

pub struct ChatService {
    pub visible: bool,
}

impl EventEmitter<ChatToggled> for ChatService {}

struct GlobalChatService(Entity<ChatService>);
impl Global for GlobalChatService {}

impl ChatService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { visible: true }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalChatService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalChatService(service.clone()));
        service
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;
        cx.emit(ChatToggled);
        cx.notify();
    }
}
