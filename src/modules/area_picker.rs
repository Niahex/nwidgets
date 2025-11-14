use gpui::{Context, Window, div, prelude::*, rgb, px};

pub struct AreaPicker {
    active: bool,
}

impl AreaPicker {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            active: false,
        }
    }
}

impl Render for AreaPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.active {
            return div();
        }

        div()
            .absolute()
            .size_full()
            .bg(rgb(0x000000).with_alpha(0.5))
            .child(
                div()
                    .absolute()
                    .top(px(100.))
                    .left(px(100.))
                    .w(px(200.))
                    .h(px(150.))
                    .border_2()
                    .border_color(rgb(0xff0000))
                    .border_dashed()
            )
    }
}
