use crate::components::TextInput;
use crate::services::{AiProvider, AiService, Message as AiMessage};
use crate::theme::*;
use gpui::*;
use gpui::prelude::FluentBuilder;

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
    ai_service: AiService,
    current_provider: AiProvider,
    is_loading: bool,
    show_settings: bool,
    openai_key_input: String,
    gemini_key_input: String,
}

impl AiChat {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "Type a message..."));

        Self {
            messages: Vec::new(),
            input,
            focus_handle: cx.focus_handle(),
            ai_service: AiService::new(),
            current_provider: AiProvider::ChatGPT,
            is_loading: false,
            show_settings: false,
            openai_key_input: String::new(),
            gemini_key_input: String::new(),
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

        // Check if API key is set for current provider
        let has_key = match self.current_provider {
            AiProvider::ChatGPT => self.ai_service.has_openai_key(),
            AiProvider::Gemini => self.ai_service.has_gemini_key(),
        };

        if !has_key {
            self.add_message(
                MessageRole::Assistant,
                format!("Please set your {} API key in settings first.", self.current_provider.name()),
                cx,
            );
            self.show_settings = true;
            cx.notify();
            return;
        }

        // Add user message
        self.add_message(MessageRole::User, text.clone(), cx);

        // Clear input
        self.input.update(cx, |input, cx| {
            input.clear(cx);
        });

        // Set loading state
        self.is_loading = true;
        cx.notify();

        // Prepare messages for AI
        let ai_messages: Vec<AiMessage> = self
            .messages
            .iter()
            .map(|msg| AiMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        let provider = self.current_provider.clone();
        let ai_service = self.ai_service.clone();

        // Spawn async task to call AI API
        cx.spawn(async move |this, mut cx| {
            let result = ai_service.send_message(provider, ai_messages).await;

            if let Some(entity) = this.upgrade() {
                let _ = cx.update_entity(&entity, |this: &mut AiChat, cx| {
                    this.is_loading = false;

                    match result {
                        Ok(response) => {
                            this.add_message(MessageRole::Assistant, response, cx);
                        }
                        Err(e) => {
                            this.add_message(
                                MessageRole::Assistant,
                                format!("Error: {}", e),
                                cx,
                            );
                        }
                    }
                });
            }
        })
        .detach();
    }

    fn on_send(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.send_message(cx);
    }

    fn on_close(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        println!("[AI_CHAT] Closing chat");
        window.remove_window();
    }

    fn toggle_provider(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_provider = match self.current_provider {
            AiProvider::ChatGPT => AiProvider::Gemini,
            AiProvider::Gemini => AiProvider::ChatGPT,
        };
        cx.notify();
    }

    fn toggle_settings(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_settings = !self.show_settings;
        cx.notify();
    }

    fn save_api_keys(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.openai_key_input.is_empty() {
            self.ai_service.set_openai_key(self.openai_key_input.clone());
        }

        if !self.gemini_key_input.is_empty() {
            self.ai_service.set_gemini_key(self.gemini_key_input.clone());
        }

        self.show_settings = false;
        cx.notify();
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
                            .child(div().text_sm().text_color(rgb(FROST1)).child(icon))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(SNOW1))
                                    .child(name),
                            ),
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
                                    .child(REFRESH),
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
                                    .child(CLIPBOARD),
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
                                    .child(EDIT),
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
                                    .child(CODE),
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
                                    .child(CLOSE),
                            ),
                    ),
            )
            .child(
                // Message content
                div().w_full().bg(rgb(POLAR2)).rounded_b_lg().p_4().child(
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
            .bg(rgb(POLAR1))
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
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(SNOW0))
                                    .child("AI Chat"),
                            )
                            .child(
                                // Provider toggle button
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(POLAR2))
                                    .rounded_md()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(FROST1))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(POLAR3)))
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::toggle_provider))
                                    .child(self.current_provider.name().to_string()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                // Settings button
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
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::toggle_settings))
                                    .child("⚙"),
                            )
                            .child(
                                // Close button
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
                    ),
            )
            // Settings dialog (if shown)
            .when(self.show_settings, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(rgba(0x00000088))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(400.0))
                                .bg(rgb(POLAR0))
                                .rounded_lg()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(SNOW0))
                                        .child("API Settings"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(SNOW1))
                                                .child("OpenAI API Key"),
                                        )
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(36.0))
                                                .bg(rgb(POLAR2))
                                                .rounded_md()
                                                .px_3()
                                                .flex()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(SNOW2))
                                                        .child("sk-..."),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(SNOW1))
                                                .child("Google Gemini API Key"),
                                        )
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(36.0))
                                                .bg(rgb(POLAR2))
                                                .rounded_md()
                                                .px_3()
                                                .flex()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(SNOW2))
                                                        .child("AIza..."),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .justify_end()
                                        .child(
                                            div()
                                                .px_4()
                                                .py_2()
                                                .bg(rgb(POLAR2))
                                                .rounded_md()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(SNOW1))
                                                .cursor_pointer()
                                                .hover(|style| style.bg(rgb(POLAR3)))
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::toggle_settings))
                                                .child("Cancel"),
                                        )
                                        .child(
                                            div()
                                                .px_4()
                                                .py_2()
                                                .bg(rgb(FROST1))
                                                .rounded_md()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(POLAR0))
                                                .cursor_pointer()
                                                .hover(|style| style.bg(rgb(FROST2)))
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::save_api_keys))
                                                .child("Save"),
                                        ),
                                ),
                        ),
                )
            })
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
                    )
                    .when(self.is_loading, |this| {
                        this.child(
                            div()
                                .w_full()
                                .p_4()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(SNOW2))
                                        .child(format!("{} is thinking...", self.current_provider.name())),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(FROST1))
                                        .child("●"),
                                ),
                        )
                    }),
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
                            .child(self.input.clone()),
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
