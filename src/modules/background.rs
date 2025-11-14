use gpui::{Context, Window, div, prelude::*, rgb, px};

pub struct Background {
    enabled: bool,
}

impl Background {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }
}

impl Render for Background {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.enabled {
            return div().size_full();
        }

        div()
            .size_full()
            .bg(rgb(0x1a1a1a)) // Dark background
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .child(
                // Wallpaper placeholder
                div()
                    .size_full()
                    .bg(rgb(0x2a2a2a))
            )
            .child(
                // Desktop clock placeholder (bottom right)
                div()
                    .absolute()
                    .bottom(px(20.))
                    .right(px(20.))
                    .p_4()
                    .bg(rgb(0x333333))
                    .rounded_md()
                    .text_color(rgb(0xffffff))
                    .child("12:34")
            )
    }
}
