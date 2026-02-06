/// Common types used across the application
use gpui::*;

/// Standard window size for popups
#[allow(dead_code)]
pub const POPUP_WIDTH: Pixels = px(700.0);
#[allow(dead_code)]
pub const POPUP_HEIGHT: Pixels = px(500.0);

/// Panel dimensions
#[allow(dead_code)]
pub const PANEL_WIDTH: Pixels = px(3440.0);
#[allow(dead_code)]
pub const PANEL_HEIGHT: Pixels = px(68.0);
#[allow(dead_code)]
pub const PANEL_EXCLUSIVE_ZONE: Pixels = px(50.0);

/// Chat window dimensions
#[allow(dead_code)]
pub const CHAT_WIDTH: Pixels = px(600.0);
#[allow(dead_code)]
pub const CHAT_HEIGHT_NORMAL: i32 = 1370;
#[allow(dead_code)]
pub const CHAT_HEIGHT_FULLSCREEN: i32 = 1440;

/// Common margins
#[allow(dead_code)]
pub const MARGIN_NORMAL: (i32, i32, i32, i32) = (40, 0, 20, 10);
#[allow(dead_code)]
pub const MARGIN_FULLSCREEN: (i32, i32, i32, i32) = (0, 0, 0, 0);

/// Hidden window size (1x1 pixel)
#[allow(dead_code)]
pub const HIDDEN_SIZE: Size<Pixels> = Size {
    width: px(1.0),
    height: px(1.0),
};
