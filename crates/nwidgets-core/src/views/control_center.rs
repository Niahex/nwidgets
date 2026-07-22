use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;

pub struct ControlCenter;

impl Render for ControlCenter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x434c5e);

        div()
            .size_full()
            .flex()
            .flex_row()
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        // Coin gauche (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                        ),
                    )
                    .child(
                        // Espace au milieu (transparent)
                        div().flex_1(),
                    )
                    .child(
                        // Coin droit (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::BottomRight, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .size_full()
                    .bg(bg)
                    .flex()
                    .items_center()
                    .px_2()
                    .child(div().text_color(rgb(0x88c0d0)).child("CONTROL CENTER")),
            )

    }
}
