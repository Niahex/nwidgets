use crate::assets::Icon;
use crate::components::Button;
use crate::theme::ActiveTheme;
use crate::widgets::r#macro::types::*;
use crate::widgets::r#macro::MacroService;
use gpui::prelude::*;
use gpui::*;

actions!(r#macro, [CloseMacro]);

pub struct MacroWidget {
    macro_service: Entity<MacroService>,
    focus_handle: FocusHandle,
    editing_name: Option<uuid::Uuid>,
    name_input: String,
    speed_input: String,
    editing_macro_id: Option<uuid::Uuid>,
    selected_action_index: Option<usize>,
    show_add_action_form: bool,
    form_action_type: String,
    form_key_code: String,
    form_timestamp: String,
    form_field_focus: FormField,
    editing_zone_for_action: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
enum FormField {
    None,
    KeyCode,
    Timestamp,
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
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn render_record_button(
        &self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_recording = self.macro_service.read(cx).is_recording();
        let icon_name = if is_recording {
            "recording-recording"
        } else {
            "recording-stopped"
        };

        div()
            .px_3()
            .py_2()
            .bg(theme.surface)
            .rounded(px(6.))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    if is_recording {
                        this.macro_service.update(cx, |service, cx| {
                            service.stop_recording(cx);
                        });
                    } else {
                        let name = format!("Macro {}", chrono::Local::now().format("%H:%M:%S"));
                        this.macro_service.update(cx, |service, cx| {
                            service.start_recording(name, cx);
                        });
                    }
                }),
            )
            .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
    }

    fn render_speed_control(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let speed = self.macro_service.read(cx).playback_speed();

        div()
            .px_3()
            .py_1()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border())
            .rounded(px(4.))
            .text_sm()
            .text_color(theme.text)
            .on_scroll_wheel(
                cx.listener(move |this, event: &ScrollWheelEvent, window, cx| {
                    let delta = event.delta.pixel_delta(window.line_height());
                    let delta_y: f32 = delta.y.into();
                    let change = if delta_y > 0.0 { 0.1 } else { -0.1 };
                    this.macro_service.update(cx, |service, cx| {
                        let new_speed = (service.playback_speed() + change).clamp(0.1, 10.0);
                        service.set_playback_speed(new_speed, cx);
                        this.speed_input = format!("{:.1}", new_speed);
                    });
                }),
            )
            .child(format!("{:.1}x", speed))
    }

    fn render_macro_list(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let macros = self.macro_service.read(cx).get_macros().clone();
        let playing_id = self.macro_service.read(cx).is_playing();

        div()
            .id("macro-list-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .gap_2()
            .children(macros.into_iter().map(|macro_rec| {
                let macro_id = macro_rec.id;
                let is_playing = playing_id == Some(macro_id);

                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_3()
                    .bg(if is_playing {
                        theme.accent.opacity(0.2)
                    } else {
                        theme.surface
                    })
                    .border_1()
                    .border_color(if is_playing {
                        theme.accent
                    } else {
                        theme.border()
                    })
                    .rounded(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(macro_rec.name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} actions", macro_rec.action_count()))
                                    .child("•")
                                    .child(format!("{}ms", macro_rec.duration_ms()))
                                    .when_some(macro_rec.app_class.clone(), |this, app| {
                                        this.child("•").child(app)
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(theme.surface))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.macro_service.update(cx, |service, cx| {
                                                if is_playing {
                                                    service.stop_playback(cx);
                                                } else {
                                                    service.play_macro(macro_id, cx);
                                                }
                                            });
                                        }),
                                    )
                                    .child(Icon::new("play").size(px(16.)).color(theme.text)),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(theme.surface))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.editing_macro_id = Some(macro_id);
                                            cx.notify();
                                        }),
                                    )
                                    .child(Icon::new("edit").size(px(16.)).color(theme.text)),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(theme.red.opacity(0.2)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.macro_service.update(cx, |service, cx| {
                                                service.delete_macro(macro_id, cx);
                                            });
                                        }),
                                    )
                                    .child(Icon::new("delete").size(px(16.)).color(theme.text)),
                            ),
                    )
            }))
    }

    fn render_macro_editor(
        &mut self,
        macro_id: uuid::Uuid,
        theme: crate::theme::Theme,
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
            Some(self.render_add_action_form(macro_id, theme.clone(), cx))
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

                        for (idx, action) in macro_rec.actions.iter().enumerate() {
                            // Ajouter un délai si nécessaire
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

    fn render_add_action_form(
        &mut self,
        macro_id: uuid::Uuid,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
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
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_xs().text_color(theme.text_muted).child(
                                if self.form_action_type == "MouseClick" {
                                    "Button (0=Left, 1=Right, 2=Middle)"
                                } else if self.form_action_type == "Delay" {
                                    "Duration (ms)"
                                } else {
                                    "Key Code"
                                },
                            ))
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(if self.form_field_focus == FormField::KeyCode {
                                        theme.accent.opacity(0.2)
                                    } else {
                                        theme.bg
                                    })
                                    .border_1()
                                    .border_color(if self.form_field_focus == FormField::KeyCode {
                                        theme.accent
                                    } else {
                                        theme.border()
                                    })
                                    .rounded(px(4.))
                                    .text_xs()
                                    .text_color(theme.text)
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.form_field_focus = FormField::KeyCode;
                                            cx.notify();
                                        }),
                                    )
                                    .child(self.form_key_code.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("Timestamp (ms)"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(if self.form_field_focus == FormField::Timestamp {
                                        theme.accent.opacity(0.2)
                                    } else {
                                        theme.bg
                                    })
                                    .border_1()
                                    .border_color(
                                        if self.form_field_focus == FormField::Timestamp {
                                            theme.accent
                                        } else {
                                            theme.border()
                                        },
                                    )
                                    .rounded(px(4.))
                                    .text_xs()
                                    .text_color(theme.text)
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.form_field_focus = FormField::Timestamp;
                                            cx.notify();
                                        }),
                                    )
                                    .child(self.form_timestamp.clone()),
                            ),
                    ),
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
                                            if let Ok(btn_code) = this.form_key_code.parse::<u32>()
                                            {
                                                let btn = match btn_code {
                                                    0 => MacroMouseButton::Left,
                                                    1 => MacroMouseButton::Right,
                                                    2 => MacroMouseButton::Middle,
                                                    _ => MacroMouseButton::Left,
                                                };
                                                Some(ActionType::MouseClick(btn))
                                            } else {
                                                None
                                            }
                                        }
                                        "Delay" => {
                                            if let Ok(ms) = this.form_key_code.parse::<u64>() {
                                                Some(ActionType::Delay(ms))
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };

                                    if let Some(action_type) = action_type {
                                        if let Ok(timestamp_ms) = this.form_timestamp.parse::<u64>()
                                        {
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
                                            this.form_timestamp = "0".to_string();
                                            cx.notify();
                                        }
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
                                    this.form_timestamp = "0".to_string();
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
