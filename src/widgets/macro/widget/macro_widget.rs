use crate::assets::Icon;
use crate::components::Button;
use crate::theme::ActiveTheme;
use crate::widgets::r#macro::types::*;
use crate::widgets::r#macro::MacroService;
use gpui::prelude::*;
use gpui::*;

actions!(r#macro, [CloseMacro]);

pub struct MacroWidget {
    pub(in crate::widgets::r#macro) macro_service: Entity<MacroService>,
    pub(in crate::widgets::r#macro) focus_handle: FocusHandle,
    pub(in crate::widgets::r#macro) editing_name: Option<uuid::Uuid>,
    pub(in crate::widgets::r#macro) name_input: String,
    pub(in crate::widgets::r#macro) speed_input: String,
    pub(in crate::widgets::r#macro) editing_macro_id: Option<uuid::Uuid>,
    pub(in crate::widgets::r#macro) selected_action_index: Option<usize>,
    pub(in crate::widgets::r#macro) show_add_action_form: bool,
    pub(in crate::widgets::r#macro) form_action_type: String,
    pub(in crate::widgets::r#macro) form_key_code: String,
    pub(in crate::widgets::r#macro) form_timestamp: String,
    pub(in crate::widgets::r#macro) form_field_focus: FormField,
    pub(in crate::widgets::r#macro) editing_zone_for_action: Option<usize>,
    pub(in crate::widgets::r#macro) form_mouse_button: MacroMouseButton,
    pub(in crate::widgets::r#macro) form_mouse_click_type: MouseClickType,
    pub(in crate::widgets::r#macro) form_delay_ms: String,
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::widgets::r#macro) enum FormField {
    None,
    KeyCode,
    Timestamp,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::widgets::r#macro) enum MouseClickType {
    Quick,
    Long,
    Double,
}

impl MouseClickType {
    fn next(&self) -> Self {
        match self {
            MouseClickType::Quick => MouseClickType::Long,
            MouseClickType::Long => MouseClickType::Double,
            MouseClickType::Double => MouseClickType::Quick,
        }
    }

    fn color(&self, theme: &crate::theme::Theme) -> Hsla {
        match self {
            MouseClickType::Quick => theme.green,
            MouseClickType::Long => theme.red,
            MouseClickType::Double => theme.purple,
        }
    }
}

impl MacroWidget {
    pub fn new(cx: &mut Context<Self>, macro_service: Entity<MacroService>) -> Self {
        cx.subscribe(
            &macro_service,
            |_this, _service, _event: &MacroRecordingChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(
            &macro_service,
            |_this, _service, _event: &MacroPlayingChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(
            &macro_service,
            |_this, _service, _event: &MacroListChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        let speed = macro_service.read(cx).playback_speed();
        Self {
            macro_service,
            focus_handle: cx.focus_handle(),
            editing_name: None,
            name_input: String::new(),
            speed_input: format!("{:.1}", speed),
            editing_macro_id: None,
            selected_action_index: None,
            show_add_action_form: false,
            form_action_type: "KeyPress".to_string(),
            form_key_code: String::new(),
            form_timestamp: "0".to_string(),
            form_field_focus: FormField::None,
            editing_zone_for_action: None,
            form_mouse_button: MacroMouseButton::Left,
            form_mouse_click_type: MouseClickType::Quick,
            form_delay_ms: String::new(),
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

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

impl Render for MacroWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &CloseMacro, _window, cx| {
                this.macro_service.update(cx, |service, cx| {
                    service.toggle_window(cx);
                });
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if this.show_add_action_form && this.form_field_focus != FormField::None {
                    match this.form_field_focus {
                        FormField::KeyCode => {
                            if event.keystroke.key == "backspace" {
                                this.form_key_code.pop();
                                cx.notify();
                            } else if event.keystroke.key == "tab" {
                                this.form_field_focus = FormField::Timestamp;
                                cx.notify();
                            } else if event.keystroke.key == "escape" {
                                this.form_field_focus = FormField::None;
                                cx.notify();
                            } else if let Some(key_char) = &event.keystroke.key_char {
                                if key_char.chars().all(|c| c.is_ascii_digit()) {
                                    this.form_key_code.push_str(key_char);
                                    cx.notify();
                                }
                            }
                        }
                        FormField::Timestamp => {
                            if event.keystroke.key == "backspace" {
                                this.form_timestamp.pop();
                                cx.notify();
                            } else if event.keystroke.key == "tab" {
                                this.form_field_focus = FormField::KeyCode;
                                cx.notify();
                            } else if event.keystroke.key == "escape" {
                                this.form_field_focus = FormField::None;
                                cx.notify();
                            } else if let Some(key_char) = &event.keystroke.key_char {
                                if key_char.chars().all(|c| c.is_ascii_digit()) {
                                    this.form_timestamp.push_str(key_char);
                                    cx.notify();
                                }
                            }
                        }
                        FormField::None => {}
                    }
                } else if let Some(macro_id) = this.editing_name {
                    if event.keystroke.key == "backspace" {
                        this.name_input.pop();
                        cx.notify();
                    } else if event.keystroke.key == "enter" {
                        let new_name = this.name_input.clone();
                        this.macro_service.update(cx, |service, cx| {
                            service.rename_macro(macro_id, new_name, cx);
                        });
                        this.editing_name = None;
                        cx.notify();
                    } else if event.keystroke.key == "escape" {
                        this.editing_name = None;
                        cx.notify();
                    } else if let Some(key_char) = &event.keystroke.key_char {
                        if key_char.len() == 1 {
                            this.name_input.push_str(key_char);
                            cx.notify();
                        }
                    }
                } else if event.keystroke.key == "escape" {
                    this.macro_service.update(cx, |service, cx| {
                        service.toggle_window(cx);
                    });
                }
            }))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.bg)
            .p_4()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(self.editing_macro_id.is_none(), |this| {
                        this.child(self.render_record_button(theme.clone(), cx))
                    })
                    .when(self.editing_macro_id.is_none(), |this| {
                        this.child(self.render_speed_control(theme.clone(), cx))
                    }),
            )
            .when(self.editing_macro_id.is_none(), |this| {
                this.child(self.render_macro_list(theme.clone(), cx))
            })
            .when_some(self.editing_macro_id, |this, macro_id| {
                this.child(self.render_macro_editor(macro_id, theme.clone(), cx))
            })
    }
}
