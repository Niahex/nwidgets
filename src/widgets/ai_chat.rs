use crate::components::TextInput;
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
    input: Entity<TextInput>,
    focus_handle: FocusHandle,
}

impl AiChat {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "Type a message..."));

        Self {
            messages: Vec::new(),
            input,
            focus_handle: cx.focus_handle(),
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

    fn send_message(&mut self, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().trim().to_string();

        if text.is_empty() {
            return;
        }

        // Add user message
        self.add_message(MessageRole::User, text.clone(), cx);

        // Clear input
        self.input.update(cx, |input, cx| {
            input.clear(cx);
        });

        // TODO: Send to AI service and get response
        // For now, just echo back
        self.add_message(
            MessageRole::Assistant,
            format!("You said: {}", text),
            cx,
        );
    }

    fn on_send(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.send_message(cx);
    }

    fn on_close(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        println!("[AI_CHAT] Closing chat");
        window.remove_window();
    }

    fn render_message(&self, message: &ChatMessage) -> impl IntoElement {
        use crate::theme::icons::*;

        let (name, icon, header_bg) = match message.role {
            MessageRole::User => ("Nia", PERSON, POLAR3),
            MessageRole::Assistant => ("Assistant", ROBOT, POLAR3),
        };

        div()
            .w_full()
            .mb_4()
            .child(
                // Header with name and action buttons
                div()
                    .w_full()
                    .h(px(36.0))
                    .bg(rgb(header_bg))
                    .rounded_t_lg()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .child(
                        // Left: icon + name
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(FROST1))
                                    .child(icon)
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(SNOW1))
                                    .child(name)
                            )
                    )
                    .child(
                        // Right: action buttons
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(REFRESH)
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CLIPBOARD)
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(EDIT)
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CODE)
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(SNOW2))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR2)))
                                    .child(CLOSE)
                            )
                    )
            )
            .child(
                // Message content
                div()
                    .w_full()
                    .bg(rgb(POLAR2))
                    .rounded_b_lg()
                    .p_4()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(SNOW1))
                            .line_height(relative(1.5))
                            .child(message.content.clone()),
                    ),
            )
    }
}

impl Focusable for AiChat {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AiChat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(POLAR1))  // Darker background like the image
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            // Header
            .child(
                div()
                    .w_full()
                    .h(px(56.0))
                    .bg(rgb(POLAR0))
                    .border_b_1()
                    .border_color(rgb(POLAR3))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(SNOW0))
                            .child("AI Chat"),
                    )
                    .child(
                        div()
                            .w(px(32.0))
                            .h(px(32.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .text_lg()
                            .text_color(rgb(SNOW2))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(POLAR2)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_close))
                            .child("×"),
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
                    .h(px(64.0))
                    .bg(rgb(POLAR0))
                    .border_t_1()
                    .border_color(rgb(POLAR3))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .child(
                        div()
                            .flex_1()
                            .h(px(40.0))
                            .bg(rgb(POLAR2))
                            .rounded_lg()
                            .px_3()
                            .flex()
                            .items_center()
                            .child(self.input.clone())
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(FROST1))
                            .rounded_lg()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(POLAR0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(FROST2)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_send))
                            .child("Send"),
                    ),
            )
    }
}
