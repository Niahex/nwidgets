use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(String, &mut Window, &mut App)>;
type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    value: SharedString,
    placeholder: Option<SharedString>,
    focused: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    on_click: Option<ClickHandler>,
    focus_handle: Option<FocusHandle>,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: "".into(),
            placeholder: None,
            focused: false,
            disabled: false,
            on_change: None,
            on_click: None,
            focus_handle: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let value = self.value.clone();
        let on_change = self.on_change.clone();
        let on_click = self.on_click.clone();

        let mut element = div()
            .id(self.id)
            .flex_1()
            .px_3()
            .py_2()
            .bg(theme.surface)
            .rounded(px(4.))
            .border_1()
            .border_color(if self.focused {
                theme.accent
            } else {
                theme.border()
            })
            .text_sm()
            .when(!self.disabled, |this| this.cursor_text());

        if let Some(focus_handle) = self.focus_handle {
            element = element
                .track_focus(&focus_handle)
                .on_click(move |_, window, cx| {
                    window.focus(&focus_handle, cx);
                    if let Some(ref handler) = on_click {
                        handler(window, cx);
                    }
                });

            if let Some(on_change) = on_change {
                element = element.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let mut new_value = value.to_string();

                    if event.keystroke.key == "backspace" {
                        new_value.pop();
                        on_change(new_value, window, cx);
                    } else if let Some(key_char) = &event.keystroke.key_char {
                        if key_char.len() == 1 {
                            new_value.push_str(key_char);
                            on_change(new_value, window, cx);
                        }
                    }
                });
            }
        } else if let Some(on_click) = on_click {
            element = element.on_click(move |_, window, cx| {
                on_click(window, cx);
            });
        }

        element
            .when_some(
                self.placeholder.filter(|_| self.value.is_empty()),
                |this, placeholder| {
                    this.child(div().text_color(theme.text_muted).child(placeholder))
                },
            )
            .when(!self.value.is_empty(), |this| {
                this.child(div().text_color(theme.text).child(self.value))
            })
    }
}
