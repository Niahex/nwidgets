use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;

pub struct Launcher;

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);

        div()
            .size_full()
            .flex()
            .flex_row()
            // ── Coin concave gauche (sous la barre) ──
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                    )
                    .child(div().flex_1()),
            )
            // ── Corps du Launcher avec les coins du bas arrondis ──
            .child(
                div()
                    .w_full()
                    .size_full()
                    .bg(bg)
                    .rounded_b(px(CORNER_RADIUS))
                    .flex()
                    .items_center()
                    .px_2()
                    .child(div().text_color(rgb(0x88c0d0)).child("LAUNCHER")),
            )
            // ── Coin concave droit (sous la barre) ──
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS)).color(bg),
                    )
                    .child(div().flex_1()),
            )
    }
}
