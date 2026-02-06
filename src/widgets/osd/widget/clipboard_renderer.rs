use crate::assets::Icon;
use gpui::prelude::*;
use gpui::*;

pub fn render_clipboard(theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .flex()
        .gap_3()
        .items_center()
        .justify_center()
        .child(Icon::new("copy").size(px(20.)).color(theme.text))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child("Copied to clipboard"),
        )
}
