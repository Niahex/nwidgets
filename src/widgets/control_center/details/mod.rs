use makepad_widgets::*;

pub mod bluetooth;
pub mod network;
pub mod monitor;

pub fn live_design(cx: &mut Cx) {
    bluetooth::live_design(cx);
    network::live_design(cx);
    monitor::live_design(cx);
}
