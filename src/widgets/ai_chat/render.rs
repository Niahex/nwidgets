use crate::theme::*;
use gpui::prelude::FluentBuilder;
use gpui::*;

use super::state::AiChat;

impl Render for AiChat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(POLAR1))
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            // Header
            .child(
                div()
                    .w_full()
                    .h(px(56.0))
                    .bg(rgb(POLAR0))
                    .border_b_1()
                    .border_color(rgb(POLAR3))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(SNOW0))
                                    .child(if self.show_settings {
                                        "API Keys Settings"
                                    } else {
                                        "AI Chat"
                                    }),
                            )
                            .when(!self.show_settings, |this| {
                                this.child(
                                    // Provider toggle button
                                    div()
                                        .px_3()
                                        .py_1()
                                        .bg(rgb(POLAR2))
                                        .rounded_md()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(FROST1))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(POLAR3)))
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(Self::toggle_provider),
                                        )
                                        .child(self.current_provider.name().to_string()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                // Settings button
                                div()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_lg()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::toggle_settings),
                                    )
                                    .child("⚙"),
                            )
                            .child(
                                // Close button
                                div()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_lg()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::on_close),
                                    )
                                    .child("×"),
                            ),
                    ),
            )
            // Main content area - either settings or messages
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .when(self.show_settings, |this| {
                        // Settings view
                        this.child(self.render_settings(cx))
                    })
                    .when(!self.show_settings, |this| {
                        // Chat view
                        this.child(
                            div()
                                .w_full()
                                .h_full()
                                .p_4()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .children(
                                    self.messages
                                        .iter()
                                        .map(|msg| self.render_message(msg))
                                        .collect::<Vec<_>>(),
                                )
                                .when(self.is_loading, |this| {
                                    this.child(
                                        div()
                                            .w_full()
                                            .p_4()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().text_color(rgb(SNOW2)).child(format!(
                                                "{} is thinking...",
                                                self.current_provider.name()
                                            )))
                                            .child(div().text_sm().text_color(rgb(FROST1)).child("●")),
                                    )
                                }),
                        )
                    }),
            )
            // Import dialog (overlay when shown)
            .when(self.show_import_dialog, |this| {
                this.child(self.render_import_dialog(cx))
            })
            // Input area (only when not in settings)
            .when(!self.show_settings, |this| {
                this.child(
                    div()
                        .w_full()
                        .h(px(64.0))
                        .bg(rgb(POLAR0))
                        .border_t_1()
                        .border_color(rgb(POLAR3))
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .child(
                            div()
                                .flex_1()
                                .h(px(40.0))
                                .bg(rgb(POLAR2))
                                .rounded_lg()
                                .px_3()
                                .flex()
                                .items_center()
                                .child(self.input.clone()),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(rgb(FROST1))
                                .rounded_lg()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(POLAR0))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(FROST2)))
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_send))
                                .child("Send"),
                        ),
                )
            })
    }
}
