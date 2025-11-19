use crate::theme::*;
use gpui::*;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
}

pub struct AiChat {
    messages: Vec<ChatMessage>,
    input_text: SharedString,
}

impl AiChat {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: "".into(),
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: String, cx: &mut Context<Self>) {
        self.messages.push(ChatMessage { role, content });
        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        cx.notify();
    }

    fn on_send(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.input_text.is_empty() {
            return;
        }

        // Add user message
        let user_message = self.input_text.to_string();
        self.add_message(MessageRole::User, user_message.clone(), cx);

        // Clear input
        self.input_text = "".into();

        // TODO: Send to AI service and get response
        // For now, just echo back
        self.add_message(
            MessageRole::Assistant,
            format!("You said: {}", user_message),
            cx,
        );
    }

    fn on_close(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        println!("[AI_CHAT] Closing chat");
        window.remove_window();
    }

    fn render_message(&self, message: &ChatMessage) -> impl IntoElement {
        let (bg_color, text_color, is_user) = match message.role {
            MessageRole::User => (FROST1, POLAR0, true),
            MessageRole::Assistant => (POLAR2, SNOW0, false),
        };

        let mut container = div()
            .flex()
            .w_full()
            .mb_2();

        if is_user {
            container = container.flex_row_reverse();
        }

        container.child(
            div()
                .max_w(px(350.0))
                .bg(rgb(bg_color))
                .rounded_lg()
                .p_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_color))
                        .child(message.content.clone()),
                ),
        )
    }
}

impl Render for AiChat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(POLAR0))
            .border_r_2()
            .border_color(rgb(FROST1))
            .flex()
            .flex_col()
            // Header
            .child(
                div()
                    .w_full()
                    .h_12()
                    .bg(rgb(POLAR1))
                    .border_b_2()
                    .border_color(rgb(FROST1))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(SNOW0))
                            .child("AI Chat"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgb(POLAR2))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(SNOW0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(POLAR3)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_close))
                            .child("✕"),
                    ),
            )
            // Messages area
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(
                        self.messages
                            .iter()
                            .map(|msg| self.render_message(msg))
                            .collect::<Vec<_>>(),
                    ),
            )
            // Input area
            .child(
                div()
                    .w_full()
                    .h_16()
                    .bg(rgb(POLAR1))
                    .border_t_2()
                    .border_color(rgb(FROST1))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .child(
                        div()
                            .flex_1()
                            .h_10()
                            .bg(rgb(POLAR2))
                            .border_1()
                            .border_color(rgb(POLAR3))
                            .rounded_md()
                            .px_3()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(SNOW1))
                                    .child(if self.input_text.is_empty() {
                                        "Type a message...".to_string()
                                    } else {
                                        self.input_text.to_string()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(FROST1))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(POLAR0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(FROST2)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_send))
                            .child("Send"),
                    ),
            )
    }
}
