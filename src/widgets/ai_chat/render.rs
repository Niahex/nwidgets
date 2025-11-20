use crate::services::AiProvider;
use crate::theme::*;
use gpui::prelude::FluentBuilder;
use gpui::*;

use super::state::AiChat;
use super::types::ChatMessage;

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
                                .child(
                                    // Token counter
                                    div()
                                        .px_2()
                                        .py_1()
                                        .bg(rgb(POLAR2))
                                        .rounded_md()
                                        .text_xs()
                                        .text_color(rgb(SNOW2))
                                        .child(format!("~{} tokens", self.total_tokens())),
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
                                .children({
                                    let messages: Vec<(usize, ChatMessage)> = self.messages
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, msg)| (idx, msg.clone()))
                                        .collect();

                                    messages.iter().map(|(idx, msg)| {
                                        self.render_message(msg, *idx, cx)
                                    }).collect::<Vec<_>>()
                                })
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
                let models: Vec<(String, String)> = self.get_available_models()
                    .into_iter()
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect();

                let current_model = self.current_model.clone();
                let current_model_name = models.iter()
                    .find(|(id, _)| id == &current_model)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| current_model.clone());

                let is_gemini = matches!(self.current_provider, AiProvider::Gemini);

                this.child(
                    div()
                        .w_full()
                        .bg(rgb(POLAR0))
                        .border_t_1()
                        .border_color(rgb(POLAR3))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_4()
                        // Input with integrated send button
                        .child(
                            div()
                                .w_full()
                                .h(px(40.0))
                                .bg(rgb(POLAR2))
                                .rounded_lg()
                                .px_3()
                                .flex()
                                .items_center()
                                .gap_2()
                                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                                    if event.keystroke.key.as_str() == "enter" {
                                        this.send_message(cx);
                                    }
                                }))
                                .child(
                                    div()
                                        .flex_1()
                                        .child(self.input.clone())
                                )
                                .child(
                                    // Send icon button
                                    div()
                                        .w(px(32.0))
                                        .h(px(32.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_md()
                                        .text_color(rgb(FROST1))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(POLAR3)))
                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_send))
                                        .child(""), // Nerd Font send icon
                                ),
                        )
                        // Bottom row: model selector + search button (if Gemini) + context info
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .relative()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            // Model selector button
                                            div()
                                                .relative()
                                                .child(
                                                    div()
                                                        .px_3()
                                                        .py_1()
                                                        .bg(rgb(POLAR2))
                                                        .rounded_lg()
                                                        .text_xs()
                                                        .text_color(rgb(SNOW2))
                                                        .cursor_pointer()
                                                        .hover(|style| style.bg(rgb(POLAR3)))
                                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::toggle_model_dropdown))
                                                        .child(format!("{} ▼", &current_model_name)),
                                        )
                                        .when(self.show_model_dropdown, |this| {
                                            this.child(
                                                div()
                                                    .absolute()
                                                    .bottom(px(32.0))
                                                    .left(px(0.0))
                                                    .min_w(px(200.0))
                                                    .bg(rgb(POLAR0))
                                                    .border_1()
                                                    .border_color(rgb(POLAR3))
                                                    .rounded_lg()
                                                    .shadow_lg()
                                                    .py_1()
                                                    .children(
                                                        models.iter().map(|(model_id, model_name)| {
                                                            let model_id_clone = model_id.clone();
                                                            let is_current = model_id == &current_model;
                                                            div()
                                                                .w_full()
                                                                .px_3()
                                                                .py_2()
                                                                .text_sm()
                                                                .text_color(if is_current {
                                                                    rgb(FROST1)
                                                                } else {
                                                                    rgb(SNOW2)
                                                                })
                                                                .cursor_pointer()
                                                                .hover(|style| style.bg(rgb(POLAR2)))
                                                                .on_mouse_down(
                                                                    gpui::MouseButton::Left,
                                                                    cx.listener(move |this, e, w, cx| {
                                                                        this.select_model(model_id_clone.clone(), e, w, cx);
                                                                    }),
                                                                )
                                                                .child(model_name.clone())
                                                        }).collect::<Vec<_>>()
                                                    )
                                            )
                                        })
                                )
                                .when(is_gemini, |this| {
                                    // Search button (only for Gemini)
                                    this.child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .bg(if self.use_search { rgb(FROST1) } else { rgb(POLAR2) })
                                            .rounded_lg()
                                            .text_xs()
                                            .text_color(if self.use_search { rgb(POLAR0) } else { rgb(SNOW2) })
                                            .cursor_pointer()
                                            .hover(|style| {
                                                if self.use_search {
                                                    style.bg(rgb(FROST2))
                                                } else {
                                                    style.bg(rgb(POLAR3))
                                                }
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::toggle_search))
                                            .child(if self.use_search { " Search" } else { " Search" }),
                                    )
                                })
                                )
                                .child(
                                    // Context info (right side)
                                    div()
                                        .px_2()
                                        .py_1()
                                        .text_xs()
                                        .text_color(rgb(SNOW2))
                                        .child({
                                            let context_msgs = if self.messages.len() > self.context_limit {
                                                self.context_limit
                                            } else {
                                                self.messages.len()
                                            };
                                            format!("{}/{} msgs", context_msgs, self.messages.len())
                                        })
                                )
                        )
                )
            })
    }
}
