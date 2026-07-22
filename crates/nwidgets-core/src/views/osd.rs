use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;


pub struct OsdView;

impl Render for OsdView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                        // Espace au milieu (transparent)
                        div().flex_1(),
                    )
                    .child(
                        // Coin concave inverse gauche bas (lié au bas de l'écran)
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
                    .rounded_t(px(CORNER_RADIUS))
                    .flex()
                    .items_center()
                    .px_2()
                    .child(div().text_color(rgb(0x88c0d0)).child("OSD")),
            )
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        // Espace au milieu (transparent)
                        div().flex_1(),
                    )
                    .child(
                        // Coin concave inverse droit bas (lié au bas de l'écran)
                        div().flex_none().child(
                            Corner::new(CornerPosition::BottomLeft, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
    }
}

/// Global wrapper for the OSD window manager.
pub struct OsdManager(pub std::cell::RefCell<nwidgets_osd::OsdWindow>);

impl OsdManager {
    pub fn new() -> Self {
        Self(std::cell::RefCell::new(nwidgets_osd::OsdWindow::new()))
    }
}

impl Global for OsdManager {}
