use gpui::*;

use crate::widgets::control_center::CloseControlCenter;
use crate::widgets::launcher::{Backspace, Down, Launch, Quit, Up};

pub fn bind_all(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("up", Up, None),
        KeyBinding::new("down", Down, None),
        KeyBinding::new("enter", Launch, None),
        KeyBinding::new("escape", Quit, None),
        KeyBinding::new("escape", CloseControlCenter, None),
    ]);
}
