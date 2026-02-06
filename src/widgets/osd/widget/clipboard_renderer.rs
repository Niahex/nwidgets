use crate::assets::Icon;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

pub fn render_clipboard(cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();

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
