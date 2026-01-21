/// Common types used across the application

use gpui::*;

/// Standard window size for popups
pub const POPUP_WIDTH: Pixels = px(700.0);
pub const POPUP_HEIGHT: Pixels = px(500.0);

/// Panel dimensions
pub const PANEL_WIDTH: Pixels = px(3440.0);
pub const PANEL_HEIGHT: Pixels = px(68.0);
pub const PANEL_EXCLUSIVE_ZONE: Pixels = px(50.0);

/// Chat window dimensions
pub const CHAT_WIDTH: Pixels = px(600.0);
pub const CHAT_HEIGHT_NORMAL: i32 = 1370;
pub const CHAT_HEIGHT_FULLSCREEN: i32 = 1440;

/// Common margins
pub const MARGIN_NORMAL: (i32, i32, i32, i32) = (40, 0, 20, 10);
pub const MARGIN_FULLSCREEN: (i32, i32, i32, i32) = (0, 0, 0, 0);

/// Hidden window size (1x1 pixel)
pub const HIDDEN_SIZE: Size<Pixels> = Size {
    width: px(1.0),
    height: px(1.0),
};
