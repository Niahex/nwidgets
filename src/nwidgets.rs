use gpui::*;

pub fn run(cx: &mut App) {
    crate::app::keys::bind_all(cx);
    
    let (clipboard, _osd) = crate::app::init::initialize_all(cx);
    let (chat_service, launcher_service, notif_service) = crate::app::init::get_globals(cx);
    
    crate::widgets::panel::window::open(cx);
    crate::widgets::chat::window::open(cx);
    crate::widgets::jisig::window::open(cx);
    crate::widgets::launcher::window::open(cx, launcher_service.clone(), clipboard);
    
    crate::app::subscriptions::setup_all(cx, chat_service, launcher_service, notif_service);
    
    cx.activate(true);
}
