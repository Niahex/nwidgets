use gpui::*;

pub struct Slider {
    value: u8,
    on_change: Box<dyn Fn(u8, &mut Window, &mut App) + 'static>,
}

impl Slider {
    pub fn new(value: u8, on_change: impl Fn(u8, &mut Window, &mut App) + 'static) -> Self {
        Self {
            value,
            on_change: Box::new(on_change),
        }
    }

    pub fn render(self, theme_bg: Hsla, theme_fg: Hsla) -> impl IntoElement {
        let value = self.value;
        let on_change = self.on_change;

        div()
            .flex_1()
            .h(px(6.))
            .bg(theme_bg)
            .rounded_full()
            .relative()
            .child(
                div()
                    .w(relative(value as f32 / 100.0))
                    .h_full()
                    .bg(theme_fg)
                    .rounded_full()
            )
            .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, window, cx| {
                // Get bounds from prepaint
                let position = event.position;
                // We'll handle this in a simpler way with click zones
                cx.stop_propagation();
            })
    }
}
