use gpui::*;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Theme {
    // Backgrounds
    pub bg: Hsla,      // polar0 - 0x2e3440
    pub surface: Hsla, // polar1 - 0x3b4252
    pub overlay: Hsla, // polar2 - 0x434c5e
    pub hover: Hsla,   // polar3 - 0x4c566a

    // Text
    pub text: Hsla,        // snow3 - 0xeceff4
    pub text_muted: Hsla,  // snow2 - 0xd8dee9
    pub text_bright: Hsla, // snow1 - 0xe5e9f0

    // Accents
    pub accent: Hsla,     // frost1 - 0x88c0d0
    pub accent_alt: Hsla, // frost3 - 0x8fbcbb

    // Status colors
    pub error: Hsla,   // red - 0xbf616a
    pub success: Hsla, // green - 0xa3be8c
    pub warning: Hsla, // yellow - 0xebcb8b

    // Special
    pub white: Hsla,         // 0xFFFFFF
    pub systray_hover: Hsla, // 0x313244 (catppuccin)
}

impl Global for Theme {}

impl Theme {
    pub fn nord_dark() -> Self {
        Self {
            bg: rgb(0x2e3440).into(),
            surface: rgb(0x3b4252).into(),
            overlay: rgb(0x434c5e).into(),
            hover: rgb(0x4c566a).into(),
            text: rgb(0xeceff4).into(),
            text_muted: rgb(0xd8dee9).into(),
            text_bright: rgb(0xe5e9f0).into(),
            accent: rgb(0x88c0d0).into(),
            accent_alt: rgb(0x8fbcbb).into(),
            error: rgb(0xbf616a).into(),
            success: rgb(0xa3be8c).into(),
            warning: rgb(0xebcb8b).into(),
            white: rgb(0xFFFFFF).into(),
            systray_hover: rgb(0x313244).into(),
        }
    }
}
