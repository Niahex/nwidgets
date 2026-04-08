use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    value: SharedString,
    placeholder: Option<SharedString>,
    on_change: Option<Box<dyn Fn(String, &mut Window, &mut App)>>,
    focus_handle: FocusHandle,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, focus_handle: FocusHandle) -> Self {
        Self {
            id: id.into(),
            value: "".into(),
            placeholder: None,
            on_change: None,
            focus_handle,
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

    pub fn on_change(mut self, handler: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let focus_handle = self.focus_handle.clone();
        let value = self.value.clone();
        let on_change = self.on_change;

        div()
            .id(self.id)
            .track_focus(&focus_handle)
            .px_3()
            .py_2()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border())
            .rounded(px(4.))
            .text_sm()
            .text_color(theme.text)
            .cursor_text()
            .on_click(move |_, window, cx| {
                window.focus(&focus_handle, cx);
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if let Some(ref handler) = on_change {
                    let mut new_value = value.to_string();

                    if event.keystroke.key == "backspace" {
                        new_value.pop();
                        handler(new_value, window, cx);
                    } else if let Some(key_char) = &event.keystroke.key_char {
                        if key_char.len() == 1 {
                            new_value.push_str(key_char);
                            handler(new_value, window, cx);
                        }
                    }
                }
            })
            .when_some(
                self.placeholder.filter(|_| self.value.is_empty()),
                |this, placeholder| {
                    this.child(div().text_color(theme.text_muted).child(placeholder))
                },
            )
            .when(!self.value.is_empty(), |this| this.child(self.value))
    }
}
