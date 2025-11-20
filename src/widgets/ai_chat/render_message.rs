use crate::theme::*;
use gpui::prelude::FluentBuilder;
use gpui::*;

use super::state::AiChat;
use super::types::{ChatMessage, MessageRole};

impl AiChat {
    pub fn render_message(&mut self, message: &ChatMessage, message_index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::theme::icons::*;

        let (name, icon, header_bg) = match message.role {
            MessageRole::User => ("Nia", PERSON, POLAR3),
            MessageRole::Assistant => ("Assistant", ROBOT, POLAR3),
        };

        let is_assistant = matches!(message.role, MessageRole::Assistant);
        let content_clone = message.content.clone();

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
                            // Refresh button (only for assistant messages)
                            .when(is_assistant, |this| {
                                this.child(
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
                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                            this.regenerate_message(message_index, cx);
                                        }))
                                        .child(REFRESH),
                                )
                            })
                            // Copy button
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
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                        this.copy_message(content_clone.clone(), cx);
                                    }))
                                    .child(CLIPBOARD),
                            )
                            // Edit button (only for user messages)
                            .when(!is_assistant, |this| {
                                this.child(
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
                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                            this.edit_message(message_index, cx);
                                        }))
                                        .child(EDIT),
                                )
                            })
                            // Delete button
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
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                        this.delete_message(message_index, cx);
                                    }))
                                    .child(CLOSE),
                            ),
                    ),
            )
            .child({
                // Message content
                let content_bg = div().w_full().bg(rgb(POLAR2)).rounded_b_lg().p_4();

                // Check if this message is being edited
                if self.editing_message_index == Some(message_index) {
                    // Show edit input with save/cancel buttons
                    content_bg.child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                // Edit input
                                div()
                                    .w_full()
                                    .child(self.edit_input.clone())
                            )
                            .child(
                                // Save/Cancel buttons
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .bg(rgb(FROST1))
                                            .rounded_md()
                                            .text_sm()
                                            .text_color(rgb(POLAR0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(FROST2)))
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.save_edit(cx);
                                            }))
                                            .child("✓ Save")
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .bg(rgb(POLAR3))
                                            .rounded_md()
                                            .text_sm()
                                            .text_color(rgb(SNOW2))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(POLAR2)))
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.cancel_edit(cx);
                                            }))
                                            .child("✕ Cancel")
                                    )
                            )
                    )
                } else {
                    // Show markdown rendered content
                    content_bg.child(super::markdown::render_markdown(&message.content))
                }
            })
    }
}
