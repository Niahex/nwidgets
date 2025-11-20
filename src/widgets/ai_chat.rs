use crate::components::TextInput;
use crate::services::{AiProvider, AiService, Message as AiMessage};
use crate::theme::*;
use gpui::prelude::FluentBuilder;
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
    ai_service: AiService,
    current_provider: AiProvider,
    is_loading: bool,
    show_settings: bool,
    new_openai_key_input: Entity<TextInput>,
    new_gemini_key_input: Entity<TextInput>,
    show_import_dialog: bool,
    import_provider: AiProvider,
    bulk_keys_input: Entity<TextInput>,
    import_status: String,
    is_validating: bool,
}

impl AiChat {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "Type a message..."));
        let new_openai_key_input = cx.new(|cx| TextInput::new(cx, "sk-..."));
        let new_gemini_key_input = cx.new(|cx| TextInput::new(cx, "AIza..."));
        let bulk_keys_input = cx.new(|cx| TextInput::new(cx, "Paste keys here (one per line)..."));

        Self {
            messages: Vec::new(),
            input,
            focus_handle: cx.focus_handle(),
            ai_service: AiService::new(),
            current_provider: AiProvider::ChatGPT,
            is_loading: false,
            show_settings: false,
            new_openai_key_input,
            new_gemini_key_input,
            show_import_dialog: false,
            import_provider: AiProvider::ChatGPT,
            bulk_keys_input,
            import_status: String::new(),
            is_validating: false,
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
                format!(
                    "Please set your {} API key in settings first.",
                    self.current_provider.name()
                ),
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
        let mut ai_service = self.ai_service.clone();

        // Spawn async task to call AI API
        cx.spawn(async move |this, cx| {
            let result = ai_service.send_message(provider, ai_messages).await;

            if let Some(entity) = this.upgrade() {
                let _ = cx.update_entity(&entity, |this: &mut AiChat, cx| {
                    this.is_loading = false;

                    match result {
                        Ok(response) => {
                            this.add_message(MessageRole::Assistant, response, cx);
                        }
                        Err(e) => {
                            this.add_message(MessageRole::Assistant, format!("Error: {}", e), cx);
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

    fn toggle_provider(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.current_provider = match self.current_provider {
            AiProvider::ChatGPT => AiProvider::Gemini,
            AiProvider::Gemini => AiProvider::ChatGPT,
        };
        cx.notify();
    }

    fn toggle_settings(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = !self.show_settings;
        cx.notify();
    }

    fn add_openai_key(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = self.new_openai_key_input.read(cx).text().trim().to_string();
        if !key.is_empty() {
            self.ai_service.get_config_mut().openai_keys.add_key(key);
            let _ = self.ai_service.save_config();
            self.new_openai_key_input
                .update(cx, |input, cx| input.clear(cx));
            cx.notify();
        }
    }

    fn remove_openai_key(&mut self, index: usize, cx: &mut Context<Self>) {
        self.ai_service
            .get_config_mut()
            .openai_keys
            .remove_key(index);
        let _ = self.ai_service.save_config();
        cx.notify();
    }

    fn add_gemini_key(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = self.new_gemini_key_input.read(cx).text().trim().to_string();
        if !key.is_empty() {
            self.ai_service.get_config_mut().gemini_keys.add_key(key);
            let _ = self.ai_service.save_config();
            self.new_gemini_key_input
                .update(cx, |input, cx| input.clear(cx));
            cx.notify();
        }
    }

    fn remove_gemini_key(&mut self, index: usize, cx: &mut Context<Self>) {
        self.ai_service
            .get_config_mut()
            .gemini_keys
            .remove_key(index);
        let _ = self.ai_service.save_config();
        cx.notify();
    }

    fn close_settings(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = false;
        cx.notify();
    }

    fn open_import_dialog(
        &mut self,
        provider: AiProvider,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.import_provider = provider;
        self.show_import_dialog = true;
        self.import_status.clear();
        self.bulk_keys_input.update(cx, |input, cx| input.clear(cx));
        cx.notify();
    }

    fn close_import_dialog(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_import_dialog = false;
        cx.notify();
    }

    fn validate_and_import_keys(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let keys_text = self.bulk_keys_input.read(cx).text().to_string();
        let keys: Vec<String> = keys_text
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if keys.is_empty() {
            self.import_status = "No keys to import".to_string();
            cx.notify();
            return;
        }

        self.is_validating = true;
        self.import_status = format!("Validating {} key(s)...", keys.len());
        cx.notify();

        let provider = self.import_provider.clone();

        // Spawn async validation task
        cx.spawn(async move |this, cx| {
            use crate::services::AiService;

            let mut valid_keys = Vec::new();
            let mut invalid_count = 0;

            for (i, key) in keys.iter().enumerate() {
                // Update status
                let _ = cx.update_entity(&this.upgrade().unwrap(), |this: &mut AiChat, cx| {
                    this.import_status = format!(
                        "Validating key {}/{}...",
                        i + 1,
                        keys.len()
                    );
                    cx.notify();
                });

                // Validate the key in a separate thread to avoid blocking
                let key_clone = key.clone();
                let provider_clone = provider.clone();
                let is_valid = std::thread::spawn(move || {
                    match provider_clone {
                        AiProvider::ChatGPT => AiService::validate_openai_key(&key_clone),
                        AiProvider::Gemini => AiService::validate_gemini_key(&key_clone),
                    }
                })
                .join()
                .unwrap_or(false);

                if is_valid {
                    valid_keys.push(key.clone());
                } else {
                    invalid_count += 1;
                }
            }

            // Add valid keys and update status
            if let Some(entity) = this.upgrade() {
                let _ = cx.update_entity(&entity, |this: &mut AiChat, cx| {
                    // Add all valid keys
                    for key in valid_keys.iter() {
                        match provider {
                            AiProvider::ChatGPT => {
                                this.ai_service.get_config_mut().openai_keys.add_key(key.clone());
                            }
                            AiProvider::Gemini => {
                                this.ai_service.get_config_mut().gemini_keys.add_key(key.clone());
                            }
                        }
                    }

                    // Save config
                    let _ = this.ai_service.save_config();

                    // Update status
                    this.is_validating = false;
                    this.import_status = format!(
                        "✓ {} valid key(s) added, {} invalid",
                        valid_keys.len(),
                        invalid_count
                    );
                    cx.notify();
                });
            }
        })
        .detach();
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
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::toggle_provider),
                                    )
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
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::toggle_settings),
                                    )
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
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(Self::on_close),
                                    )
                                    .child("×"),
                            ),
                    ),
            )
            // Settings dialog (if shown)
            .when(self.show_settings, |this| {
                let openai_keys = self.ai_service.get_config().openai_keys.get_keys().clone();
                let gemini_keys = self.ai_service.get_config().gemini_keys.get_keys().clone();

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
                                .w(px(500.0))
                                .max_h(px(600.0))
                                .bg(rgb(POLAR0))
                                .rounded_lg()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_4()
                                // Header
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(SNOW0))
                                        .child("API Keys Management"),
                                )
                                // OpenAI Section
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(SNOW1))
                                                        .child("OpenAI API Keys"),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .px_2()
                                                                .py_1()
                                                                .bg(rgb(POLAR3))
                                                                .rounded_md()
                                                                .text_xs()
                                                                .text_color(rgb(FROST1))
                                                                .cursor_pointer()
                                                                .hover(|style| style.bg(rgb(FROST1)).text_color(rgb(POLAR0)))
                                                                .on_mouse_down(
                                                                    gpui::MouseButton::Left,
                                                                    cx.listener(move |this, e, w, cx| {
                                                                        this.open_import_dialog(AiProvider::ChatGPT, e, w, cx);
                                                                    }),
                                                                )
                                                                .child("Import")
                                                        )
                                                        .child(
                                                            div().text_xs().text_color(rgb(SNOW2)).child(
                                                                format!("{} key(s)", openai_keys.len()),
                                                            )
                                                        ),
                                                ),
                                        )
                                        // Existing keys
                                        .children(
                                            openai_keys
                                                .iter()
                                                .enumerate()
                                                .map(|(index, key)| {
                                                    let masked_key = if key.len() > 12 {
                                                        format!(
                                                            "{}...{}",
                                                            &key[..6],
                                                            &key[key.len() - 4..]
                                                        )
                                                    } else {
                                                        "****".to_string()
                                                    };

                                                    div()
                                                        .w_full()
                                                        .h(px(32.0))
                                                        .bg(rgb(POLAR2))
                                                        .rounded_md()
                                                        .px_3()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .text_color(rgb(SNOW2))
                                                                .child(masked_key),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(24.0))
                                                                .h(px(24.0))
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .rounded_md()
                                                                .text_base()
                                                                .text_color(rgb(RED))
                                                                .cursor_pointer()
                                                                .hover(|style| {
                                                                    style.bg(rgb(POLAR3))
                                                                })
                                                                .on_mouse_down(
                                                                    gpui::MouseButton::Left,
                                                                    cx.listener(
                                                                        move |this, _, _, cx| {
                                                                            this.remove_openai_key(
                                                                                index, cx,
                                                                            );
                                                                        },
                                                                    ),
                                                                )
                                                                .child("×"),
                                                        )
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                        // New key input
                                        .child(
                                            div()
                                                .w_full()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .h(px(36.0))
                                                        .bg(rgb(POLAR2))
                                                        .rounded_md()
                                                        .px_3()
                                                        .flex()
                                                        .items_center()
                                                        .child(self.new_openai_key_input.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(36.0))
                                                        .h(px(36.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bg(rgb(FROST1))
                                                        .rounded_md()
                                                        .text_lg()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(POLAR0))
                                                        .cursor_pointer()
                                                        .hover(|style| style.bg(rgb(FROST2)))
                                                        .on_mouse_down(
                                                            gpui::MouseButton::Left,
                                                            cx.listener(Self::add_openai_key),
                                                        )
                                                        .child("+"),
                                                ),
                                        ),
                                )
                                // Gemini Section
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(SNOW1))
                                                        .child("Google Gemini API Keys"),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .px_2()
                                                                .py_1()
                                                                .bg(rgb(POLAR3))
                                                                .rounded_md()
                                                                .text_xs()
                                                                .text_color(rgb(FROST1))
                                                                .cursor_pointer()
                                                                .hover(|style| style.bg(rgb(FROST1)).text_color(rgb(POLAR0)))
                                                                .on_mouse_down(
                                                                    gpui::MouseButton::Left,
                                                                    cx.listener(move |this, e, w, cx| {
                                                                        this.open_import_dialog(AiProvider::Gemini, e, w, cx);
                                                                    }),
                                                                )
                                                                .child("Import")
                                                        )
                                                        .child(
                                                            div().text_xs().text_color(rgb(SNOW2)).child(
                                                                format!("{} key(s)", gemini_keys.len()),
                                                            )
                                                        ),
                                                ),
                                        )
                                        // Existing keys
                                        .children(
                                            gemini_keys
                                                .iter()
                                                .enumerate()
                                                .map(|(index, key)| {
                                                    let masked_key = if key.len() > 12 {
                                                        format!(
                                                            "{}...{}",
                                                            &key[..6],
                                                            &key[key.len() - 4..]
                                                        )
                                                    } else {
                                                        "****".to_string()
                                                    };

                                                    div()
                                                        .w_full()
                                                        .h(px(32.0))
                                                        .bg(rgb(POLAR2))
                                                        .rounded_md()
                                                        .px_3()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .text_color(rgb(SNOW2))
                                                                .child(masked_key),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(24.0))
                                                                .h(px(24.0))
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .rounded_md()
                                                                .text_base()
                                                                .text_color(rgb(RED))
                                                                .cursor_pointer()
                                                                .hover(|style| {
                                                                    style.bg(rgb(POLAR3))
                                                                })
                                                                .on_mouse_down(
                                                                    gpui::MouseButton::Left,
                                                                    cx.listener(
                                                                        move |this, _, _, cx| {
                                                                            this.remove_gemini_key(
                                                                                index, cx,
                                                                            );
                                                                        },
                                                                    ),
                                                                )
                                                                .child("×"),
                                                        )
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                        // New key input
                                        .child(
                                            div()
                                                .w_full()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .h(px(36.0))
                                                        .bg(rgb(POLAR2))
                                                        .rounded_md()
                                                        .px_3()
                                                        .flex()
                                                        .items_center()
                                                        .child(self.new_gemini_key_input.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(36.0))
                                                        .h(px(36.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bg(rgb(FROST1))
                                                        .rounded_md()
                                                        .text_lg()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(POLAR0))
                                                        .cursor_pointer()
                                                        .hover(|style| style.bg(rgb(FROST2)))
                                                        .on_mouse_down(
                                                            gpui::MouseButton::Left,
                                                            cx.listener(Self::add_gemini_key),
                                                        )
                                                        .child("+"),
                                                ),
                                        ),
                                )
                                // Close button
                                .child(
                                    div().flex().justify_end().child(
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
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(Self::close_settings),
                                            )
                                            .child("Close"),
                                    ),
                                ),
                        ),
                )
            })
            // Import dialog (if shown)
            .when(self.show_import_dialog, |this| {
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
                                .w(px(500.0))
                                .bg(rgb(POLAR0))
                                .rounded_lg()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_4()
                                // Header
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(SNOW0))
                                        .child(format!("Import {} Keys", self.import_provider.name())),
                                )
                                // Description
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(SNOW2))
                                        .child("Paste multiple API keys below (one per line). Each key will be validated before being added."),
                                )
                                // Textarea for keys
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(200.0))
                                        .bg(rgb(POLAR2))
                                        .rounded_md()
                                        .p_3()
                                        .child(self.bulk_keys_input.clone()),
                                )
                                // Status message
                                .when(!self.import_status.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .w_full()
                                            .p_3()
                                            .bg(rgb(POLAR2))
                                            .rounded_md()
                                            .text_sm()
                                            .text_color(if self.import_status.starts_with("✓") {
                                                rgb(GREEN)
                                            } else {
                                                rgb(SNOW2)
                                            })
                                            .child(self.import_status.clone()),
                                    )
                                })
                                // Buttons
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
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::close_import_dialog))
                                                .child("Cancel"),
                                        )
                                        .child(
                                            div()
                                                .px_4()
                                                .py_2()
                                                .bg(if self.is_validating { rgb(POLAR3) } else { rgb(FROST1) })
                                                .rounded_md()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(if self.is_validating { rgb(SNOW2) } else { rgb(POLAR0) })
                                                .when(!self.is_validating, |this| {
                                                    this.cursor_pointer()
                                                        .hover(|style| style.bg(rgb(FROST2)))
                                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::validate_and_import_keys))
                                                })
                                                .child(if self.is_validating { "Validating..." } else { "Validate & Import" }),
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
                                .child(div().text_sm().text_color(rgb(SNOW2)).child(format!(
                                    "{} is thinking...",
                                    self.current_provider.name()
                                )))
                                .child(div().text_sm().text_color(rgb(FROST1)).child("●")),
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
