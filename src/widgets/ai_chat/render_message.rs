use crate::theme::*;
use gpui::*;

use super::state::AiChat;
use super::types::{ChatMessage, MessageRole};

impl AiChat {
    pub fn render_message(&self, message: &ChatMessage) -> impl IntoElement {
        use crate::theme::icons::*;

        let (name, icon, header_bg) = match message.role {
            MessageRole::User => ("Nia", PERSON, POLAR3),
            MessageRole::Assistant => ("Assistant", ROBOT, POLAR3),
        };

        div()
            .w_full()
            .mb_4()
            .child(
                // Header with name and action buttons
                div()
                    .w_full()
                    .h(px(36.0))
                    .bg(rgb(header_bg))
                    .rounded_t_lg()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .child(
                        // Left: icon + name
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().text_color(rgb(FROST1)).child(icon))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(SNOW1))
                                    .child(name),
                            ),
                    )
                    .child(
                        // Right: action buttons
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(REFRESH),
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CLIPBOARD),
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(EDIT),
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CODE),
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CLOSE),
                            ),
                    ),
            )
            .child(
                // Message content
                div().w_full().bg(rgb(POLAR2)).rounded_b_lg().p_4().child(
                    div()
                        .text_sm()
                        .text_color(rgb(SNOW1))
                        .line_height(relative(1.5))
                        .child(message.content.clone()),
                ),
            )
    }
}
