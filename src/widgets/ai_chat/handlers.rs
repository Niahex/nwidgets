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

        // ========== TOKEN OPTIMIZATION ==========

        // 1. Limiter le contexte aux derniers N messages
        let start_idx = if self.messages.len() > self.context_limit {
            self.messages.len() - self.context_limit
        } else {
            0
        };

        let mut context_messages: Vec<ChatMessage> = self.messages
            .iter()
            .skip(start_idx)
            .cloned()
            .collect();

        // 2. Auto-summarize: Résumer les messages exclus si activé
        let mut optimized_messages = if self.auto_summarize && start_idx > 0 {
            // Créer un résumé des messages exclus
            let excluded_messages: Vec<ChatMessage> = self.messages
                .iter()
                .take(start_idx)
                .cloned()
                .collect();

            let summary = super::optimizer::summarize_messages(&excluded_messages);

            // Commencer avec le résumé, puis ajouter les messages récents
            let mut result = vec![summary];
            result.extend(context_messages);
            result
        } else {
            context_messages
        };

        // 3. Supprimer la redondance (formules de politesse, répétitions)
        optimized_messages = super::optimizer::remove_redundancy(&optimized_messages);

        // 4. Compresser les anciens messages si activé (garde les 3 derniers intacts)
        if self.compress_context && optimized_messages.len() > 5 {
            let keep_recent = 3;
            let compress_until = optimized_messages.len().saturating_sub(keep_recent);

            for i in 0..compress_until {
                if let Some(msg) = optimized_messages.get_mut(i) {
                    *msg = super::optimizer::compress_message(msg);
                }
            }
        }

        // 5. Générer le system prompt optimisé
        let system_prompt = super::optimizer::generate_system_prompt(
            self.concise_mode,
            self.use_search
        );

        // Convertir en format API avec system prompt en premier
        let mut ai_messages: Vec<AiMessage> = vec![
            AiMessage {
                role: "system".to_string(),
                content: system_prompt,
            }
        ];

        // Ajouter les messages optimisés
        ai_messages.extend(
            optimized_messages.iter().map(|msg| AiMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
        );

        let provider = self.current_provider.clone();
        let model = self.current_model.clone();
        let mut ai_service = self.ai_service.clone();

        // Spawn async task to call AI API
        cx.spawn(async move |this, cx| {
            let result = ai_service.send_message(provider, model, ai_messages).await;

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
            AiProvider::ChatGPT => {
                self.current_model = "gemini-2.5-flash".to_string();
                AiProvider::Gemini
            },
            AiProvider::Gemini => {
                self.current_model = "gpt-4o-mini".to_string();
                AiProvider::ChatGPT
            },
        };
        cx.notify();
    }

    pub fn toggle_model_dropdown(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_model_dropdown = !self.show_model_dropdown;
        cx.notify();
    }

    pub fn select_model(
        &mut self,
        model: String,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.current_model = model;
        self.show_model_dropdown = false;
        cx.notify();
    }

    pub fn toggle_search(
        &mut self,
        _: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.use_search = !self.use_search;
        cx.notify();
    }

    pub fn get_available_models(&self) -> Vec<(&str, &str)> {
        match self.current_provider {
            AiProvider::ChatGPT => vec![
                ("gpt-4o", "GPT-4o"),
                ("gpt-4o-mini", "GPT-4o Mini"),
                ("gpt-4-turbo", "GPT-4 Turbo"),
                ("gpt-4", "GPT-4"),
                ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
            ],
            AiProvider::Gemini => vec![
                ("gemini-2.5-flash", "Gemini 2.5 Flash (Rapide)"),
                ("gemini-2.5-pro", "Gemini 2.5 Pro (Puissant)"),
            ],
        }
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

    // ========== Message Actions ==========

    /// Copy message content to clipboard
    pub fn copy_message(&mut self, content: String, cx: &mut Context<Self>) {
        let item = gpui::ClipboardItem::new_string(content);
        cx.write_to_clipboard(item);
    }

    /// Delete a message at the given index
    pub fn delete_message(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.messages.len() {
            self.messages.remove(index);
            cx.notify();
        }
    }

    /// Enter edit mode for a user message
    pub fn edit_message(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.messages.len() {
            return;
        }

        // Verify it's a user message
        if let Some(msg) = self.messages.get(index) {
            if !matches!(msg.role, MessageRole::User) {
                return; // Only edit user messages
            }

            // Put the message content in edit input
            let content = msg.content.clone();
            self.edit_input.update(cx, |input, cx| {
                input.set_text(content, cx);
            });

            // Enter edit mode
            self.editing_message_index = Some(index);
            cx.notify();
        }
    }

    /// Save the edited message
    pub fn save_edit(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.editing_message_index {
            let new_content = self.edit_input.read(cx).text().trim().to_string();

            if !new_content.is_empty() && index < self.messages.len() {
                // Update message content
                self.messages[index].content = new_content;

                // Remove all messages after this one (they're now outdated)
                self.messages.truncate(index + 1);
            }

            // Exit edit mode
            self.editing_message_index = None;
            self.edit_input.update(cx, |input, cx| input.clear(cx));
            cx.notify();
        }
    }

    /// Cancel editing
    pub fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.editing_message_index = None;
        self.edit_input.update(cx, |input, cx| input.clear(cx));
        cx.notify();
    }

    /// Regenerate assistant response (resend the conversation up to this point)
    pub fn regenerate_message(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.messages.len() {
            return;
        }

        // Verify it's an assistant message
        if let Some(msg) = self.messages.get(index) {
            if !matches!(msg.role, MessageRole::Assistant) {
                return;
            }

            // Remove this assistant message and all subsequent messages
            self.messages.truncate(index);

            // Resend the last user message
            if let Some(last_user_msg) = self.messages.iter().rev().find(|m| matches!(m.role, MessageRole::User)) {
                let user_text = last_user_msg.content.clone();

                // Remove the last user message from history (we'll re-add it in send_message)
                if let Some(last_idx) = self.messages.iter().rposition(|m| matches!(m.role, MessageRole::User)) {
                    self.messages.remove(last_idx);
                }

                // Put it in the input and send
                self.input.update(cx, |input, cx| {
                    input.set_text(user_text, cx);
                });

                self.send_message(cx);
            }
        }
    }
}
