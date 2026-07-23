use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};

const CORNER_RADIUS: f32 = 12.0;

actions!(chat, [CloseChat]);

pub struct Chat {
    pub focus_handle: FocusHandle,
}

impl Chat {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for Chat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x434c5e);
        let frost_border = rgb(0x88c0d0).opacity(0.3);

        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_this, _action: &CloseChat, _window, cx| {
                // Notifie la fermeture du Chat
                cx.emit(CloseChat);
            }))
            .size_full()
            .flex()
            .flex_row()
            // ── Main Chat Container ──
            .child(
                div()
                    .w_full()
                    .size_full()
                    .bg(bg)
                    .border_b_1()
                    .border_l_1()
                    .border_color(frost_border)
                    .flex()
                    .items_center()
                    .px_2()
                    .child(div().text_color(rgb(0x88c0d0)).child("CHAT")),
            )
            // ── Right Concave Corners Column ──
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        // Top-Left concave corner (under top bar)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS))
                                .color(bg)
                                .border_color(frost_border),
                        ),
                    )
                    .child(
                        // Vertical border line in the right column
                        div().flex_1().flex().justify_start().child(
                            div().w(px(1.0)).h_full().bg(frost_border),
                        ),
                    )
                    .child(
                        // Bottom-Left concave corner
                        div().flex_none().child(
                            Corner::new(CornerPosition::BottomLeft, px(CORNER_RADIUS))
                                .color(bg)
                                .border_color(frost_border),
                        ),
                    ),
            )
    }
}

impl EventEmitter<CloseChat> for Chat {}
