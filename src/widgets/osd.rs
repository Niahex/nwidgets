use gpui::*;

pub struct OsdWidget {
    message: SharedString,
    value: f32,
}

impl OsdWidget {
    pub fn new(message: impl Into<SharedString>, value: f32) -> Self {
        Self {
            message: message.into(),
            value: value.clamp(0.0, 1.0),
        }
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(rgb(0x1e1e1e))
            .rounded_lg()
            .shadow_lg()
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(0xffffff))
                    .child(self.message.clone())
            )
            .child(
                div()
                    .flex()
                    .h(px(8.))
                    .w(px(200.))
                    .bg(rgb(0x3e3e3e))
                    .rounded_full()
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.value))
                            .bg(rgb(0x0078d4))
                            .rounded_full()
                    )
            )
    }
}
