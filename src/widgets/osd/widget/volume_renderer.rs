use crate::assets::Icon;
use crate::components::{Slider, SliderState};
use gpui::prelude::*;
use gpui::*;

pub fn render_volume(
    icon_name: &str,
    displayed_volume: f32,
    volume_slider: &Entity<SliderState>,
    theme: &crate::theme::Theme,
) -> impl IntoElement {
    let display_val = ((displayed_volume / 5.0).round() * 5.0) as u8;

    div()
        .flex()
        .gap_3()
        .items_center()
        .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
        .child(div().w(px(240.)).child(Slider::new(volume_slider).readonly(true)))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(format!("{display_val}")),
        )
}
