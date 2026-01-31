use makepad_widgets::*;

pub mod button;
pub mod toggle;
pub mod slider;
pub mod circular_progress;

pub fn live_design(cx: &mut Cx) {
    button::live_design(cx);
    toggle::live_design(cx);
    slider::live_design(cx);
    circular_progress::live_design(cx);
}
