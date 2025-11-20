use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeyManager {
    keys: Vec<String>,
    current_index: usize,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_key(&mut self, key: String) {
        if !key.trim().is_empty() && !self.keys.contains(&key) {
            self.keys.push(key);
        }
    }

    pub fn remove_key(&mut self, index: usize) {
        if index < self.keys.len() {
            self.keys.remove(index);
            if self.current_index >= self.keys.len() && self.current_index > 0 {
                self.current_index = self.keys.len() - 1;
            }
        }
    }

    pub fn get_current_key(&self) -> Option<&String> {
        self.keys.get(self.current_index)
    }

    pub fn get_keys(&self) -> &Vec<String> {
        &self.keys
    }

    pub fn rotate_to_next(&mut self) -> bool {
        if self.keys.is_empty() {
            return false;
        }
        self.current_index = (self.current_index + 1) % self.keys.len();
        true
    }

    pub fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConfig {
    pub openai_keys: ApiKeyManager,
    pub gemini_keys: ApiKeyManager,
}

impl AiConfig {
    pub fn new() -> Self {
        Self {
            openai_keys: ApiKeyManager::new(),
            gemini_keys: ApiKeyManager::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AiConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        println!("[AI_CONFIG] Saved to {:?}", config_path);
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home.join(".config/nwidgets/ai_config.json"))
    }
}

pub struct AiService {
    client: reqwest::Client,
    config: AiConfig,
}

impl Clone for AiService {
    fn clone(&self) -> Self {
        Self {
            client: reqwest::Client::new(),
            config: self.config.clone(),
        }
    }
}

impl AiService {
    pub fn new() -> Self {
        let config = AiConfig::load().unwrap_or_else(|e| {
            println!("[AI_SERVICE] Failed to load config: {}, using empty config", e);
            AiConfig::new()
        });

        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub fn get_config(&self) -> &AiConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut AiConfig {
        &mut self.config
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save()
    }

    pub fn has_openai_key(&self) -> bool {
        self.config.openai_keys.has_keys()
    }

    pub fn has_gemini_key(&self) -> bool {
        self.config.gemini_keys.has_keys()
    }

    pub async fn send_message(
        &mut self,
        provider: AiProvider,
        messages: Vec<Message>,
    ) -> Result<String> {
        match provider {
            AiProvider::ChatGPT => self.send_to_openai(messages).await,
            AiProvider::Gemini => self.send_to_gemini(messages).await,
        }
    }

    async fn send_to_openai(&mut self, messages: Vec<Message>) -> Result<String> {
        let max_retries = self.config.openai_keys.get_keys().len().max(1);
        let mut last_error = None;

        for _attempt in 0..max_retries {
            let api_key = self
                .config
                .openai_keys
                .get_current_key()
                .ok_or_else(|| anyhow!("No OpenAI API key available"))?
                .clone();

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
                messages: messages.clone(),
            };

            let response = self
                .client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();

            if !status.is_success() {
                let error_text = response.text().await?;

                // Check if it's a rate limit error (429)
                if status.as_u16() == 429 {
                    println!("[AI_SERVICE] Rate limit hit on OpenAI key, rotating to next...");
                    last_error = Some(anyhow!("Rate limit: {}", error_text));
                    self.config.openai_keys.rotate_to_next();
                    continue;
                }

                return Err(anyhow!("OpenAI API error {}: {}", status, error_text));
            }

            let ai_response: OpenAiResponse = response.json().await?;

            return ai_response
                .choices
                .first()
                .map(|choice| choice.message.content.clone())
                .ok_or_else(|| anyhow!("No response from OpenAI"));
        }

        // If we exhausted all retries
        Err(last_error.unwrap_or_else(|| anyhow!("All OpenAI API keys exhausted")))
    }

    async fn send_to_gemini(&mut self, messages: Vec<Message>) -> Result<String> {
        let max_retries = self.config.gemini_keys.get_keys().len().max(1);
        let mut last_error = None;

        for _attempt in 0..max_retries {
            let api_key = self
                .config
                .gemini_keys
                .get_current_key()
                .ok_or_else(|| anyhow!("No Gemini API key available"))?
                .clone();

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
                .iter()
                .filter_map(|msg| {
                    let role = match msg.role.as_str() {
                        "system" => return None, // Gemini doesn't support system messages directly
                        "user" => "user",
                        "assistant" => "model",
                        _ => "user",
                    };

                    Some(GeminiContent {
                        role: role.to_string(),
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                        }],
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

            let status = response.status();

            if !status.is_success() {
                let error_text = response.text().await?;

                // Check if it's a rate limit error (429)
                if status.as_u16() == 429 {
                    println!("[AI_SERVICE] Rate limit hit on Gemini key, rotating to next...");
                    last_error = Some(anyhow!("Rate limit: {}", error_text));
                    self.config.gemini_keys.rotate_to_next();
                    continue;
                }

                return Err(anyhow!("Gemini API error {}: {}", status, error_text));
            }

            let ai_response: GeminiResponse = response.json().await?;

            return ai_response
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.clone())
                .ok_or_else(|| anyhow!("No response from Gemini"));
        }

        // If we exhausted all retries
        Err(last_error.unwrap_or_else(|| anyhow!("All Gemini API keys exhausted")))
    }
}

impl Default for AiService {
    fn default() -> Self {
        Self::new()
    }
}
