use crate::services::AiProvider;
use crate::theme::*;
use gpui::prelude::FluentBuilder;
use gpui::*;

use super::state::AiChat;

impl AiChat {
    pub fn render_settings(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let openai_keys = self.ai_service.get_config().openai_keys.get_keys().clone();
        let gemini_keys = self.ai_service.get_config().gemini_keys.get_keys().clone();

        div()
            .w_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            // OpenAI Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(SNOW1))
                                    .child("OpenAI API Keys"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(POLAR3))
                                            .rounded_md()
                                            .text_xs()
                                            .text_color(rgb(FROST1))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(FROST1)).text_color(rgb(POLAR0)))
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(move |this, e, w, cx| {
                                                    this.open_import_dialog(AiProvider::ChatGPT, e, w, cx);
                                                }),
                                            )
                                            .child("Import")
                                    )
                                    .child(
                                        div().text_xs().text_color(rgb(SNOW2)).child(
                                            format!("{} key(s)", openai_keys.len()),
                                        )
                                    ),
                            ),
                    )
                    // Existing keys
                    .children(
                        openai_keys
                            .iter()
                            .enumerate()
                            .map(|(index, key)| {
                                let masked_key = if key.len() > 12 {
                                    format!(
                                        "{}...{}",
                                        &key[..6],
                                        &key[key.len() - 4..]
                                    )
                                } else {
                                    "****".to_string()
                                };

                                div()
                                    .w_full()
                                    .h(px(32.0))
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .px_3()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(SNOW2))
                                            .child(masked_key),
                                    )
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .h(px(24.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded_md()
                                            .text_base()
                                            .text_color(rgb(RED))
                                            .cursor_pointer()
                                            .hover(|style| {
                                                style.bg(rgb(POLAR3))
                                            })
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(
                                                    move |this, _, _, cx| {
                                                        this.remove_openai_key(
                                                            index, cx,
                                                        );
                                                    },
                                                ),
                                            )
                                            .child("×"),
                                    )
                            })
                            .collect::<Vec<_>>(),
                    )
                    // New key input
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(36.0))
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .px_3()
                                    .flex()
                                    .items_center()
                                    .child(self.new_openai_key_input.clone()),
                            )
                            .child(
                                div()
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(rgb(FROST1))
                                    .rounded_md()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(POLAR0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(FROST2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::add_openai_key),
                                    )
                                    .child("+"),
                            ),
                    ),
            )
            // Gemini Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(SNOW1))
                                    .child("Google Gemini API Keys"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(POLAR3))
                                            .rounded_md()
                                            .text_xs()
                                            .text_color(rgb(FROST1))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(FROST1)).text_color(rgb(POLAR0)))
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(move |this, e, w, cx| {
                                                    this.open_import_dialog(AiProvider::Gemini, e, w, cx);
                                                }),
                                            )
                                            .child("Import")
                                    )
                                    .child(
                                        div().text_xs().text_color(rgb(SNOW2)).child(
                                            format!("{} key(s)", gemini_keys.len()),
                                        )
                                    ),
                            ),
                    )
                    // Existing keys
                    .children(
                        gemini_keys
                            .iter()
                            .enumerate()
                            .map(|(index, key)| {
                                let masked_key = if key.len() > 12 {
                                    format!(
                                        "{}...{}",
                                        &key[..6],
                                        &key[key.len() - 4..]
                                    )
                                } else {
                                    "****".to_string()
                                };

                                div()
                                    .w_full()
                                    .h(px(32.0))
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .px_3()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(SNOW2))
                                            .child(masked_key),
                                    )
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .h(px(24.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded_md()
                                            .text_base()
                                            .text_color(rgb(RED))
                                            .cursor_pointer()
                                            .hover(|style| {
                                                style.bg(rgb(POLAR3))
                                            })
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(
                                                    move |this, _, _, cx| {
                                                        this.remove_gemini_key(
                                                            index, cx,
                                                        );
                                                    },
                                                ),
                                            )
                                            .child("×"),
                                    )
                            })
                            .collect::<Vec<_>>(),
                    )
                    // New key input
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(36.0))
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .px_3()
                                    .flex()
                                    .items_center()
                                    .child(self.new_gemini_key_input.clone()),
                            )
                            .child(
                                div()
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(rgb(FROST1))
                                    .rounded_md()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(POLAR0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(FROST2)))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::add_gemini_key),
                                    )
                                    .child("+"),
                            ),
                    ),
            )
    }

    pub fn render_import_dialog(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(rgba(0x00000088))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(450.0))
                    .bg(rgb(POLAR0))
                    .rounded_lg()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
                    // Header
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(SNOW0))
                            .child(format!("Import {} Keys", self.import_provider.name())),
                    )
                    // Description
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(SNOW2))
                            .child("Paste multiple API keys below (one per line). Each key will be validated before being added."),
                    )
                    // Textarea for keys
                    .child(
                        div()
                            .w_full()
                            .h(px(200.0))
                            .bg(rgb(POLAR2))
                            .rounded_md()
                            .p_3()
                            .child(self.bulk_keys_input.clone()),
                    )
                    // Status message
                    .when(!self.import_status.is_empty(), |this| {
                        this.child(
                            div()
                                .w_full()
                                .p_3()
                                .bg(rgb(POLAR2))
                                .rounded_md()
                                .text_sm()
                                .text_color(if self.import_status.starts_with("✓") {
                                    rgb(GREEN)
                                } else {
                                    rgb(SNOW2)
                                })
                                .child(self.import_status.clone()),
                        )
                    })
                    // Buttons
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(SNOW1))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR3)))
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::close_import_dialog))
                                    .child("Cancel"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(if self.is_validating { rgb(POLAR3) } else { rgb(FROST1) })
                                    .rounded_md()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(if self.is_validating { rgb(SNOW2) } else { rgb(POLAR0) })
                                    .when(!self.is_validating, |this| {
                                        this.cursor_pointer()
                                            .hover(|style| style.bg(rgb(FROST2)))
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::validate_and_import_keys))
                                    })
                                    .child(if self.is_validating { "Validating..." } else { "Validate & Import" }),
                            ),
                    ),
            )
    }
}
