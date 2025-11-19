use gpui::*;

pub struct TextInput {
    text: String,
    placeholder: SharedString,
    focus_handle: FocusHandle,
    on_enter: Option<Box<dyn Fn(&str) + 'static>>,
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            text: String::new(),
            placeholder: placeholder.into(),
            focus_handle: cx.focus_handle(),
            on_enter: None,
        }
    }

    pub fn on_enter(mut self, callback: impl Fn(&str) + 'static) -> Self {
        self.on_enter = Some(Box::new(callback));
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.text = text;
        cx.notify();
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.text.clear();
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        match key {
            "enter" => {
                if let Some(on_enter) = &self.on_enter {
                    on_enter(&self.text);
                }
            }
            "backspace" => {
                self.text.pop();
                cx.notify();
            }
            "space" => {
                self.text.push(' ');
                cx.notify();
            }
            _ => {
                // Add any single character
                if key.len() == 1 {
                    self.text.push_str(key);
                    cx.notify();
                }
            }
        }
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::theme::*;

        div()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(
                div()
                    .text_sm()
                    .text_color(if self.text.is_empty() {
                        rgb(SNOW0)
                    } else {
                        rgb(SNOW2)
                    })
                    .child(if self.text.is_empty() {
                        self.placeholder.to_string()
                    } else {
                        self.text.clone()
                    }),
            )
    }
}
