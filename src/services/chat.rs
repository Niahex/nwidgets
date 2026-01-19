use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

pub struct ChatToggled;
pub struct ChatPinToggled;
pub struct ChatNavigate {
    pub url: String,
}

pub struct ChatService {
    pub visible: bool,
    pub pinned: bool,
}

impl EventEmitter<ChatToggled> for ChatService {}
impl EventEmitter<ChatPinToggled> for ChatService {}
impl EventEmitter<ChatNavigate> for ChatService {}

struct GlobalChatService(Entity<ChatService>);
impl Global for GlobalChatService {}

impl ChatService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            pinned: false,
        }
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

        // Close control center when chat opens
        if self.visible {
            let cc = crate::services::control_center::ControlCenterService::global(cx);
            cc.update(cx, |cc, cx| {
                if cc.is_visible() {
                    cc.toggle(cx);
                }
            });
        }

        cx.emit(ChatToggled);
        cx.notify();
    }

    pub fn toggle_pin(&mut self, cx: &mut Context<Self>) {
        self.pinned = !self.pinned;
        cx.emit(ChatPinToggled);
        cx.notify();
    }

    pub fn navigate(&mut self, url: String, cx: &mut Context<Self>) {
        cx.emit(ChatNavigate { url });
    }
}
