// src/theme/colors.rs

use once_cell::sync::Lazy;

/// A struct to represent a color and allow manipulations.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    hex: &'static str,
}

impl Color {
    /// Returns the color as a CSS hex string (e.g., "#RRGGBB").
    pub fn to_hex_string(self) -> String {
        format!("#{}", self.hex)
    }

    /// Returns the color as a CSS RGBA string with opacity (e.g., "rgba(r, g, b, a)").
    /// GTK's CSS engine prefers rgba() for opacity.
    pub fn with_opacity(self, percentage: u8) -> String {
        let r = u8::from_str_radix(&self.hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&self.hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&self.hex[4..6], 16).unwrap_or(0);
        let alpha = (percentage.clamp(0, 100) as f32) / 100.0;
        format!("rgba({}, {}, {}, {})", r, g, b, alpha)
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Colors {
    // Polar Night (Dark backgrounds)
    pub polar0: Color,
    pub polar1: Color,
    pub polar2: Color,
    pub polar3: Color,

    // Snow Storm (Light text)
    pub snow0: Color,
    pub snow1: Color,
    pub snow2: Color,

    // Frost (Blues)
    pub frost0: Color,
    pub frost1: Color,
    pub frost2: Color,
    pub frost3: Color,

    // Aurora (Accent colors)
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub purple: Color,
}

impl Colors {
    fn new() -> Self {
        Self {
            // Polar Night
            polar0: Color { hex: "2e3440" },
            polar1: Color { hex: "3b4252" },
            polar2: Color { hex: "434c5e" },
            polar3: Color { hex: "4c566a" },
            // Snow Storm
            snow0: Color { hex: "d8dee9" },
            snow1: Color { hex: "e5e9f0" },
            snow2: Color { hex: "eceff4" },
            // Frost
            frost0: Color { hex: "8fbcbb" },
            frost1: Color { hex: "88c0d0" },
            frost2: Color { hex: "81a1c1" },
            frost3: Color { hex: "5e81ac" },
            // Aurora
            red: Color { hex: "bf616a" },
            orange: Color { hex: "d08770" },
            yellow: Color { hex: "ebcb8b" },
            green: Color { hex: "a3be8c" },
            purple: Color { hex: "b48ead" },
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self::new()
    }
}

pub static COLORS: Lazy<Colors> = Lazy::new(Colors::new);
