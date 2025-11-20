use crate::services::{AiProvider, AiService, Message as AiMessage};
use gpui::*;

use super::state::AiChat;
use super::types::{ChatMessage, MessageRole};

impl AiChat {
    pub fn add_message(&mut self, role: MessageRole, content: String, cx: &mut Context<Self>) {
        self.messages.push(ChatMessage { role, content });
        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        cx.notify();
    }

    pub fn send_message(&mut self, cx: &mut Context<Self>) {
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

    pub fn on_send(&mut self, _: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.send_message(cx);
    }

    pub fn on_close(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        println!("[AI_CHAT] Closing chat");
        window.remove_window();
    }

    pub fn toggle_provider(
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

    pub fn toggle_settings(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = !self.show_settings;
        cx.notify();
    }

    pub fn add_openai_key(
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

    pub fn remove_openai_key(&mut self, index: usize, cx: &mut Context<Self>) {
        self.ai_service
            .get_config_mut()
            .openai_keys
            .remove_key(index);
        let _ = self.ai_service.save_config();
        cx.notify();
    }

    pub fn add_gemini_key(
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

    pub fn remove_gemini_key(&mut self, index: usize, cx: &mut Context<Self>) {
        self.ai_service
            .get_config_mut()
            .gemini_keys
            .remove_key(index);
        let _ = self.ai_service.save_config();
        cx.notify();
    }

    pub fn close_settings(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = false;
        cx.notify();
    }

    pub fn open_import_dialog(
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

    pub fn close_import_dialog(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_import_dialog = false;
        cx.notify();
    }

    pub fn validate_and_import_keys(
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
}
