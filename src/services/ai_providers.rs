use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AiProvider {
    ChatGPT,
    Gemini,
}

impl AiProvider {
    pub fn name(&self) -> &str {
        match self {
            AiProvider::ChatGPT => "ChatGPT",
            AiProvider::Gemini => "Gemini",
        }
    }
}

pub struct AiService {
    client: reqwest::Client,
    openai_key: Option<String>,
    gemini_key: Option<String>,
}

impl Clone for AiService {
    fn clone(&self) -> Self {
        Self {
            client: reqwest::Client::new(),
            openai_key: self.openai_key.clone(),
            gemini_key: self.gemini_key.clone(),
        }
    }
}

impl AiService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            openai_key: None,
            gemini_key: None,
        }
    }

    pub fn set_openai_key(&mut self, key: String) {
        self.openai_key = Some(key);
    }

    pub fn set_gemini_key(&mut self, key: String) {
        self.gemini_key = Some(key);
    }

    pub fn get_openai_key(&self) -> Option<String> {
        self.openai_key.clone()
    }

    pub fn get_gemini_key(&self) -> Option<String> {
        self.gemini_key.clone()
    }

    pub fn has_openai_key(&self) -> bool {
        self.openai_key.is_some()
    }

    pub fn has_gemini_key(&self) -> bool {
        self.gemini_key.is_some()
    }

    pub async fn send_message(
        &self,
        provider: AiProvider,
        messages: Vec<Message>,
    ) -> Result<String> {
        match provider {
            AiProvider::ChatGPT => self.send_to_openai(messages).await,
            AiProvider::Gemini => self.send_to_gemini(messages).await,
        }
    }

    async fn send_to_openai(&self, messages: Vec<Message>) -> Result<String> {
        let api_key = self
            .openai_key
            .as_ref()
            .ok_or_else(|| anyhow!("OpenAI API key not set"))?;

        #[derive(Serialize)]
        struct OpenAiRequest {
            model: String,
            messages: Vec<Message>,
        }

        #[derive(Deserialize)]
        struct OpenAiResponse {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: MessageResponse,
        }

        #[derive(Deserialize)]
        struct MessageResponse {
            content: String,
        }

        let request = OpenAiRequest {
            model: "gpt-4o-mini".to_string(),
            messages,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error {}: {}", status, error_text));
        }

        let ai_response: OpenAiResponse = response.json().await?;

        ai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No response from OpenAI"))
    }

    async fn send_to_gemini(&self, messages: Vec<Message>) -> Result<String> {
        let api_key = self
            .gemini_key
            .as_ref()
            .ok_or_else(|| anyhow!("Gemini API key not set"))?;

        // Convert messages to Gemini format
        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<GeminiContent>,
        }

        #[derive(Serialize)]
        struct GeminiContent {
            role: String,
            parts: Vec<GeminiPart>,
        }

        #[derive(Serialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }

        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiContentResponse,
        }

        #[derive(Deserialize)]
        struct GeminiContentResponse {
            parts: Vec<GeminiPartResponse>,
        }

        #[derive(Deserialize)]
        struct GeminiPartResponse {
            text: String,
        }

        // Convert from OpenAI format (system/user/assistant) to Gemini format (user/model)
        let contents: Vec<GeminiContent> = messages
            .into_iter()
            .filter_map(|msg| {
                let role = match msg.role.as_str() {
                    "system" => return None, // Gemini doesn't support system messages directly
                    "user" => "user",
                    "assistant" => "model",
                    _ => "user",
                };

                Some(GeminiContent {
                    role: role.to_string(),
                    parts: vec![GeminiPart { text: msg.content }],
                })
            })
            .collect();

        let request = GeminiRequest { contents };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error {}: {}", status, error_text));
        }

        let ai_response: GeminiResponse = response.json().await?;

        ai_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| anyhow!("No response from Gemini"))
    }
}

impl Default for AiService {
    fn default() -> Self {
        Self::new()
    }
}
