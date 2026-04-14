use crate::assets::Icon;
use crate::theme::Theme;
use crate::widgets::r#macro::types::ActionType;
use gpui::prelude::*;
use gpui::*;

impl super::MacroWidget {
    pub(super) fn render_macro_editor(
        &mut self,
        macro_id: uuid::Uuid,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let macro_opt = self
            .macro_service
            .read(cx)
            .get_macros()
            .iter()
            .find(|m| m.id == macro_id)
            .cloned();

        let Some(macro_rec) = macro_opt else {
            return div().child("Macro not found");
        };

        let add_form = if self.show_add_action_form {
            Some(deferred(self.render_add_action_form(
                macro_id,
                theme.clone(),
                cx,
            )))
        } else {
            None
        };

        let is_editing_name = self.editing_name == Some(macro_id);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text)
                                    .child("Editing: "),
                            )
                            .children(if !is_editing_name {
                                let name_for_display = macro_rec.name.clone();
                                let name_for_handler = macro_rec.name.clone();
                                Some(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.text)
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _window, cx| {
                                                this.editing_name = Some(macro_id);
                                                this.name_input = name_for_handler.clone();
                                                cx.notify();
                                            }),
                                        )
                                        .child(name_for_display),
                                )
                            } else {
                                None
                            })
                            .children(if is_editing_name {
                                Some(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .bg(theme.bg)
                                        .border_1()
                                        .border_color(theme.accent)
                                        .rounded(px(4.))
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.text)
                                        .child(self.name_input.clone()),
                                )
                            } else {
                                None
                            }),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(theme.surface)
                            .rounded(px(4.))
                            .text_xs()
                            .text_color(theme.text)
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.editing_macro_id = None;
                                    this.selected_action_index = None;
                                    this.editing_name = None;
                                    cx.notify();
                                }),
                            )
                            .child("← Back to List"),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .bg(theme.green)
                    .rounded(px(4.))
                    .text_sm()
                    .text_color(theme.bg)
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _window, cx| {
                            this.show_add_action_form = !this.show_add_action_form;
                            cx.notify();
                        }),
                    )
                    .child("+ Add Action"),
            )
            .when_some(add_form, |this, form| this.child(form))
            .child(
                div()
                    .id("macro-actions-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .gap_2()
                    .children({
                        let mut elements = Vec::new();
                        let mut prev_timestamp = 0u64;

                        for (idx, action) in macro_rec.actions.iter().take(100).enumerate() {
                            if idx > 0 {
                                let delay_ms = action.timestamp_ms.saturating_sub(prev_timestamp);
                                if delay_ms > 0 {
                                    elements.push(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .p_2()
                                            .bg(theme.surface.opacity(0.5))
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded(px(4.))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child("⏱"),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .text_color(theme.yellow)
                                                    .child(format!("Delay: {}ms", delay_ms)),
                                            )
                                            .child(
                                                div()
                                                    .p_2()
                                                    .rounded(px(4.))
                                                    .cursor_pointer()
                                                    .hover(|style| style.bg(theme.red.opacity(0.2)))
                                                    .child(
                                                        Icon::new("delete")
                                                            .size(px(16.))
                                                            .color(theme.text),
                                                    ),
                                            ),
                                    );
                                }
                            }

                            prev_timestamp = action.timestamp_ms;

                            let is_selected = self.selected_action_index == Some(idx);
                            let action_clone = action.clone();
                            let zone_for_display = action_clone.click_zone.clone();
                            let zone_for_handler = zone_for_display.clone();

                            elements.push(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .p_2()
                                    .bg(if is_selected {
                                        theme.accent.opacity(0.2)
                                    } else {
                                        theme.surface
                                    })
                                    .border_1()
                                    .border_color(if is_selected {
                                        theme.accent
                                    } else {
                                        theme.border()
                                    })
                                    .rounded(px(4.))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .flex_1()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.selected_action_index = Some(idx);

                                                    if let Some(zone) = zone_for_handler.clone() {
                                                        crate::widgets::r#macro::window::zone_viewer::open(
                                                            cx, zone,
                                                        );
                                                    } else {
                                                        crate::widgets::r#macro::window::zone_viewer::close(
                                                            cx,
                                                        );
                                                    }

                                                    cx.notify();
                                                }),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(format!("#{}", idx + 1)),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme.text)
                                                            .child(action_clone.action_type.display_name()),
                                                    )
                                                    .when_some(zone_for_display.clone(), |this, zone| {
                                                        this.child(
                                                            div().text_xs().text_color(theme.accent).child(
                                                                format!(
                                                                    "🎯 Zone: {}x{} at ({},{})",
                                                                    zone.width, zone.height, zone.x, zone.y
                                                                ),
                                                            ),
                                                        )
                                                    }),
                                            ),
                                    )
                                    .child(div().flex().flex_col().gap_1().when(
                                        matches!(
                                            action_clone.action_type,
                                            ActionType::MouseClick(_)
                                        ),
                                        |this| {
                                            this.child(
                                                div()
                                                    .p_2()
                                                    .rounded(px(4.))
                                                    .cursor_pointer()
                                                    .hover(|style| style.bg(theme.surface))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(
                                                            move |this, _, _window, _cx| {
                                                                this.editing_zone_for_action =
                                                                    Some(idx);
                                                            },
                                                        ),
                                                    )
                                                    .child(
                                                        Icon::new("target")
                                                            .size(px(16.))
                                                            .color(theme.text),
                                                    ),
                                            )
                                        },
                                    ))
                                    .child(
                                        div()
                                            .p_2()
                                            .rounded(px(4.))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(theme.red.opacity(0.2)))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    if let Some(macro_id) = this.editing_macro_id {
                                                        this.macro_service.update(
                                                            cx,
                                                            |service, cx| {
                                                                service.delete_action(
                                                                    macro_id, idx, cx,
                                                                );
                                                            },
                                                        );
                                                        if this.selected_action_index == Some(idx) {
                                                            this.selected_action_index = None;
                                                        }
                                                    }
                                                }),
                                            )
                                            .child(
                                                Icon::new("delete").size(px(16.)).color(theme.text),
                                            ),
                                    ),
                            );
                        }

                        elements
                    }),
            )
    }
}
