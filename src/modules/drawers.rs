use gpui::{Context, Window, div, prelude::*, rgb, px};

pub struct Drawers {
    visible: bool,
}

impl Drawers {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
        }
    }
}

impl Render for Drawers {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div();
        }

        div()
            .absolute()
            .size_full()
            .child(
                // Left drawer placeholder
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .h_full()
                    .w(px(300.))
                    .bg(rgb(0x2a2a2a))
                    .border_r_1()
                    .border_color(rgb(0x444444))
                    .p_4()
                    .child("Drawer Content")
            )
    }
}
