use crate::theme::Theme;
use crate::widgets::r#macro::types::*;
use gpui::prelude::*;
use gpui::*;

use super::macro_widget::MouseClickType;

impl super::MacroWidget {
    pub(super) fn render_add_action_form(
        &mut self,
        macro_id: uuid::Uuid,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_mouse_click = self.form_action_type == "MouseClick";
        let is_delay = self.form_action_type == "Delay";
        let is_key_action =
            self.form_action_type == "KeyPress" || self.form_action_type == "KeyRelease";

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.accent)
            .rounded(px(6.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Add New Action"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("Action Type"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(if self.form_action_type == "KeyPress" {
                                                theme.accent
                                            } else {
                                                theme.bg
                                            })
                                            .rounded(px(4.))
                                            .text_xs()
                                            .text_color(theme.text)
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.form_action_type = "KeyPress".to_string();
                                                    cx.notify();
                                                }),
                                            )
                                            .child("Key Press"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(if self.form_action_type == "KeyRelease" {
                                                theme.accent
                                            } else {
                                                theme.bg
                                            })
                                            .rounded(px(4.))
                                            .text_xs()
                                            .text_color(theme.text)
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.form_action_type =
                                                        "KeyRelease".to_string();
                                                    cx.notify();
                                                }),
                                            )
                                            .child("Key Release"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(if self.form_action_type == "MouseClick" {
                                                theme.accent
                                            } else {
                                                theme.bg
                                            })
                                            .rounded(px(4.))
                                            .text_xs()
                                            .text_color(theme.text)
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.form_action_type =
                                                        "MouseClick".to_string();
                                                    cx.notify();
                                                }),
                                            )
                                            .child("Mouse Click"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(if self.form_action_type == "Delay" {
                                                theme.accent
                                            } else {
                                                theme.bg
                                            })
                                            .rounded(px(4.))
                                            .text_xs()
                                            .text_color(theme.text)
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.form_action_type = "Delay".to_string();
                                                    cx.notify();
                                                }),
                                            )
                                            .child("Delay"),
                                    ),
                            ),
                    )
                    .when(is_key_action, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child("Key Code"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .bg(theme.bg)
                                        .border_1()
                                        .border_color(theme.accent)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(theme.text)
                                        .child(self.form_key_code.clone()),
                                ),
                        )
                    })
                    .when(is_mouse_click, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child("Mouse Button (click to cycle type)"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex_1()
                                                .px_3()
                                                .py_2()
                                                .bg(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Left
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.bg
                                                    },
                                                )
                                                .border_1()
                                                .border_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Left
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.border()
                                                    },
                                                )
                                                .rounded(px(4.))
                                                .text_xs()
                                                .text_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Left
                                                    ) {
                                                        theme.bg
                                                    } else {
                                                        theme.text
                                                    },
                                                )
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _window, cx| {
                                                        if matches!(
                                                            this.form_mouse_button,
                                                            MacroMouseButton::Left
                                                        ) {
                                                            this.form_mouse_click_type =
                                                                this.form_mouse_click_type.next();
                                                        } else {
                                                            this.form_mouse_button =
                                                                MacroMouseButton::Left;
                                                            this.form_mouse_click_type =
                                                                MouseClickType::Quick;
                                                        }
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(format!(
                                                    "Left {:?}",
                                                    self.form_mouse_click_type
                                                )),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .px_3()
                                                .py_2()
                                                .bg(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Middle
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.bg
                                                    },
                                                )
                                                .border_1()
                                                .border_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Middle
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.border()
                                                    },
                                                )
                                                .rounded(px(4.))
                                                .text_xs()
                                                .text_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Middle
                                                    ) {
                                                        theme.bg
                                                    } else {
                                                        theme.text
                                                    },
                                                )
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _window, cx| {
                                                        if matches!(
                                                            this.form_mouse_button,
                                                            MacroMouseButton::Middle
                                                        ) {
                                                            this.form_mouse_click_type =
                                                                this.form_mouse_click_type.next();
                                                        } else {
                                                            this.form_mouse_button =
                                                                MacroMouseButton::Middle;
                                                            this.form_mouse_click_type =
                                                                MouseClickType::Quick;
                                                        }
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(format!(
                                                    "Middle {:?}",
                                                    self.form_mouse_click_type
                                                )),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .px_3()
                                                .py_2()
                                                .bg(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Right
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.bg
                                                    },
                                                )
                                                .border_1()
                                                .border_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Right
                                                    ) {
                                                        self.form_mouse_click_type.color(&theme)
                                                    } else {
                                                        theme.border()
                                                    },
                                                )
                                                .rounded(px(4.))
                                                .text_xs()
                                                .text_color(
                                                    if matches!(
                                                        self.form_mouse_button,
                                                        MacroMouseButton::Right
                                                    ) {
                                                        theme.bg
                                                    } else {
                                                        theme.text
                                                    },
                                                )
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _window, cx| {
                                                        if matches!(
                                                            this.form_mouse_button,
                                                            MacroMouseButton::Right
                                                        ) {
                                                            this.form_mouse_click_type =
                                                                this.form_mouse_click_type.next();
                                                        } else {
                                                            this.form_mouse_button =
                                                                MacroMouseButton::Right;
                                                            this.form_mouse_click_type =
                                                                MouseClickType::Quick;
                                                        }
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(format!(
                                                    "Right {:?}",
                                                    self.form_mouse_click_type
                                                )),
                                        ),
                                ),
                        )
                    })
                    .when(is_delay, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child("Duration (ms)"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .bg(theme.bg)
                                        .border_1()
                                        .border_color(theme.accent)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(theme.text)
                                        .child(self.form_delay_ms.clone()),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
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
                                    let action_type = match this.form_action_type.as_str() {
                                        "KeyPress" => {
                                            if let Ok(code) = this.form_key_code.parse::<u32>() {
                                                Some(ActionType::KeyPress(code))
                                            } else {
                                                None
                                            }
                                        }
                                        "KeyRelease" => {
                                            if let Ok(code) = this.form_key_code.parse::<u32>() {
                                                Some(ActionType::KeyRelease(code))
                                            } else {
                                                None
                                            }
                                        }
                                        "MouseClick" => {
                                            Some(ActionType::MouseClick(this.form_mouse_button))
                                        }
                                        "Delay" => {
                                            if let Ok(ms) = this.form_delay_ms.parse::<u64>() {
                                                Some(ActionType::Delay(ms))
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };

                                    if let Some(action_type) = action_type {
                                        let timestamp_ms = 0;
                                        let action = MacroAction {
                                            timestamp_ms,
                                            action_type,
                                            click_zone: None,
                                        };
                                        this.macro_service.update(cx, |service, cx| {
                                            service.add_action(macro_id, action, cx);
                                        });
                                        this.show_add_action_form = false;
                                        this.form_key_code.clear();
                                        this.form_delay_ms.clear();
                                        cx.notify();
                                    }
                                }),
                            )
                            .child("✓ Add"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .bg(theme.red)
                            .rounded(px(4.))
                            .text_sm()
                            .text_color(theme.bg)
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.show_add_action_form = false;
                                    this.form_key_code.clear();
                                    this.form_delay_ms.clear();
                                    cx.notify();
                                }),
                            )
                            .child("✗ Cancel"),
                    ),
            )
    }
}
