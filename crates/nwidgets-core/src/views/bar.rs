use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;
const BAR_HEIGHT: f32 = 50.0;

pub struct Bar;

impl Render for Bar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // ── Bar content ──
                div()
                    .w_full()
                    .h(px(BAR_HEIGHT))
                    .bg(bg)
                    .flex()
                    .items_center()
                    .px_2()
                    .child(div().text_color(rgb(0x88c0d0)).child("nwidgets bar")),
            )
            // ── Corners sous la barre ──
            .child(
                div()
                    .w_full()
                    .h(px(CORNER_RADIUS))
                    .flex()
                    .flex_row()
                    .child(
                        // Coin gauche (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS)).color(bg),
                        ),
                    )
                    .child(
                        // Espace au milieu (transparent)
                        div().flex_1(),
                    )
                    .child(
                        // Coin droit (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
    }
}
