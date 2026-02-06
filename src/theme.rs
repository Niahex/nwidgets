use gpui::{App, Context, Global, Hsla, rgb};

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
    pub muted: Hsla,       // snow2 at 40% - 0xd8dee9

    // Accents
    pub accent: Hsla,     // frost1 - 0x88c0d0
    pub accent_alt: Hsla, // frost3 - 0x8fbcbb
    pub accent_10: Hsla,  // frost1 - 10 %
    pub accent_15: Hsla,
    pub accent_20: Hsla,
    pub accent_25: Hsla,
    pub accent_50: Hsla,
    pub accent_75: Hsla,

    // Status colors
    pub red: Hsla,    // red - 0xbf616a
    pub orange: Hsla, // orange - 0xd08770
    pub yellow: Hsla, // yellow - 0xebcb8b
    pub green: Hsla,  // green - 0xa3be8c
    pub purple: Hsla, // purple - 0xb48ead

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
            muted: Hsla::from(rgb(0xd8dee9)).opacity(0.4),
            accent: rgb(0x88c0d0).into(),
            accent_alt: rgb(0x8fbcbb).into(),
            accent_10: Hsla::from(rgb(0x8fbcbb)).opacity(0.1),
            accent_15: Hsla::from(rgb(0x8fbcbb)).opacity(0.15),
            accent_20: Hsla::from(rgb(0x8fbcbb)).opacity(0.20),
            accent_25: Hsla::from(rgb(0x8fbcbb)).opacity(0.25),
            accent_50: Hsla::from(rgb(0x8fbcbb)).opacity(0.50),
            accent_75: Hsla::from(rgb(0x8fbcbb)).opacity(0.75),
            red: rgb(0xbf616a).into(),
            orange: rgb(0xd08770).into(),
            yellow: rgb(0xebcb8b).into(),
            green: rgb(0xa3be8c).into(),
            purple: rgb(0xb48ead).into(),
            white: rgb(0xFFFFFF).into(),
            systray_hover: rgb(0x313244).into(),
        }
    }

    pub fn background(&self) -> Hsla {
        self.bg
    }

    pub fn border(&self) -> Hsla {
        self.overlay
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl<V> ActiveTheme for Context<'_, V> {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}
