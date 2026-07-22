use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;

pub struct NtfView;

impl Render for NtfView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x4c566a);

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // ── Bloc principal (Haut) avec le coin haut-gauche ──
                div()
                    .w_full()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .child(
                        // Bande externe gauche pour le coin sous la barre
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
                    .child(
                        // Corps du notification panel
                        div()
                            .w_full()
                            .h_full()
                            .bg(bg)
                            .rounded_bl(px(CORNER_RADIUS))
                            .flex()
                            .items_center()
                            .px_2()
                            .child(div().text_color(rgb(0xe5e9f0)).child("Notifications")),
                    ),
            )
            .child(
                // ── Ligne du bas : Coin concave SOUS le coin inférieur droit ──
                div()
                    .w_full()
                    .h(px(CORNER_RADIUS))
                    .flex()
                    .flex_row()
                    .child(div().flex_1()) // Espace transparent sous tout le reste du panel
                    .child(
                        // Coin concave placé en dessous du bord droit du panel
                        Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                    ),
            )
    }
}

/// Global wrapper for the notification window manager.
pub struct NtfManager(pub std::cell::RefCell<nwidgets_notification::NotificationWindow>);

impl NtfManager {
    pub fn new() -> Self {
        Self(std::cell::RefCell::new(
            nwidgets_notification::NotificationWindow::new(),
        ))
    }
}

impl Global for NtfManager {}
