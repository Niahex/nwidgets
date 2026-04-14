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
    pub(in crate::widgets::r#macro) fn next(&self) -> Self {
        match self {
            MouseClickType::Quick => MouseClickType::Long,
            MouseClickType::Long => MouseClickType::Double,
            MouseClickType::Double => MouseClickType::Quick,
        }
    }

    pub(in crate::widgets::r#macro) fn color(&self, theme: &crate::theme::Theme) -> Hsla {
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
                this.child(deferred(self.render_macro_editor(
                    macro_id,
                    theme.clone(),
                    cx,
                )))
            })
    }
}
