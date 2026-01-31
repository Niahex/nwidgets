use makepad_widgets::*;

pub mod active_window;
pub mod workspaces;
pub mod pomodoro;
pub mod mpris;
pub mod systray;
pub mod bluetooth;
pub mod network;
pub mod sink;
pub mod source;
pub mod datetime;

pub use active_window::*;
pub use workspaces::*;
pub use pomodoro::*;
pub use mpris::*;
pub use systray::*;
pub use bluetooth::*;
pub use network::*;
pub use sink::*;
pub use source::*;
pub use datetime::*;

pub fn live_design(cx: &mut Cx) {
    active_window::live_design(cx);
    workspaces::live_design(cx);
    pomodoro::live_design(cx);
    mpris::live_design(cx);
    systray::live_design(cx);
    bluetooth::live_design(cx);
    network::live_design(cx);
    sink::live_design(cx);
    source::live_design(cx);
    datetime::live_design(cx);
}
