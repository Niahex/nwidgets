use crate::components::TextInput;
use crate::services::{AiProvider, AiService};
use gpui::*;

use super::types::ChatMessage;

pub struct AiChat {
    pub messages: Vec<ChatMessage>,
    pub input: Entity<TextInput>,
    pub focus_handle: FocusHandle,
    pub ai_service: AiService,
    pub current_provider: AiProvider,
    pub is_loading: bool,
    pub show_settings: bool,
    pub new_openai_key_input: Entity<TextInput>,
    pub new_gemini_key_input: Entity<TextInput>,
    pub show_import_dialog: bool,
    pub import_provider: AiProvider,
    pub bulk_keys_input: Entity<TextInput>,
    pub import_status: String,
    pub is_validating: bool,
    pub show_model_dropdown: bool,
    pub current_model: String,
    pub use_search: bool,
    pub context_limit: usize, // Nombre de messages à garder dans le contexte
    pub concise_mode: bool, // Mode réponses courtes
    pub auto_summarize: bool, // Résumé automatique après X messages
    pub compress_context: bool, // Compression des messages anciens
    pub editing_message_index: Option<usize>, // Index du message en cours d'édition
    pub edit_input: Entity<TextInput>, // Input pour éditer les messages
}

impl AiChat {
    /// Estime le nombre de tokens dans un texte (approximation: 1 token ≈ 4 caractères)
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f32 / 4.0).ceil() as usize
    }

    /// Calcule le nombre total de tokens dans l'historique actuel
    pub fn total_tokens(&self) -> usize {
        self.messages.iter()
            .map(|msg| Self::estimate_tokens(&msg.content))
            .sum()
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "Type a message..."));
        let new_openai_key_input = cx.new(|cx| TextInput::new(cx, "sk-..."));
        let new_gemini_key_input = cx.new(|cx| TextInput::new(cx, "AIza..."));
        let bulk_keys_input = cx.new(|cx| TextInput::new(cx, "Paste keys here (one per line)..."));
        let edit_input = cx.new(|cx| TextInput::new(cx, "Edit message..."));

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
            show_model_dropdown: false,
            current_model: "gpt-4o-mini".to_string(),
            use_search: false,
            context_limit: 20, // Garder les 20 derniers messages (10 échanges)
            concise_mode: false,
            auto_summarize: true, // Activé par défaut
            compress_context: true, // Activé par défaut
            editing_message_index: None,
            edit_input,
        }
    }
}

impl Focusable for AiChat {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
