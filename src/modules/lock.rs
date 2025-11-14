use gpui::{Context, Window, div, prelude::*, rgb, px};

pub struct Lock {
    locked: bool,
}

impl Lock {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            locked: false,
        }
    }
}

impl Render for Lock {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.locked {
            return div();
        }

        div()
            .absolute()
            .size_full()
            .bg(rgb(0x000000).with_alpha(0.9))
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .p_8()
                    .bg(rgb(0x2a2a2a))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(0x444444))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .text_color(rgb(0xffffff))
                            .child("Screen Locked")
                    )
                    .child(
                        div()
                            .w(px(200.))
                            .h(px(30.))
                            .bg(rgb(0x1a1a1a))
                            .border_1()
                            .border_color(rgb(0x555555))
                            .rounded_md()
                            .p_2()
                            .child("Enter password...")
                    )
            )
    }
}
