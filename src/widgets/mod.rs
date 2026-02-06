use makepad_widgets::*;

pub mod panel;
pub mod launcher;
pub mod control_center;
pub mod notifications;
pub mod osd;
pub mod project_manager;

pub fn live_design(cx: &mut Cx) {
    panel::register_live_design(cx);
    launcher::live_design(cx);
    control_center::register_live_design(cx);
    notifications::live_design(cx);
    osd::live_design(cx);
}
