use crate::assets::Icon;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

pub fn render_capslock(enabled: bool, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let icon_name = if enabled {
        "capslock-on"
    } else {
        "capslock-off"
    };
    let text = if enabled {
        "Caps Lock On"
    } else {
        "Caps Lock Off"
    };

    div()
        .flex()
        .gap_3()
        .items_center()
        .justify_center()
        .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(text),
        )
}
