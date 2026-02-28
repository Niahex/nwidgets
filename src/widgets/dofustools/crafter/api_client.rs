use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::types::{DofusItem, DofusRecipe};

const API_BASE: &str = "https://api.dofusdb.fr";

pub struct DofusApiClient {
    client: reqwest::Client,
}

impl DofusApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Search items by name
    pub async fn search_items(&self, query: &str, limit: usize) -> Result<Vec<DofusItem>> {
        // DofusDB API: name is an object with language keys (en, fr, de, es, pt)
        // Use regex search on name.fr with case-insensitive option
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/items?name.fr[$regex]=.*{}.*&name.fr[$options]=i&$limit={}",
            API_BASE, encoded_query, limit
        );
        log::info!("[API] Searching with URL: {}", url);
        self.fetch_list(&url).await
    }

    /// Get item by ID
    pub async fn get_item(&self, item_id: i32) -> Result<DofusItem> {
        let url = format!("{}/items/{}", API_BASE, item_id);
        self.fetch_one(&url).await
    }

    /// Get recipe for an item
    pub async fn get_recipe(&self, item_id: i32) -> Result<Option<DofusRecipe>> {
        let url = format!("{}/recipes?resultId={}", API_BASE, item_id);
        let recipes: Vec<DofusRecipe> = self.fetch_list(&url).await?;
        Ok(recipes.into_iter().next())
    }

    /// Get multiple items by IDs
    pub async fn get_items_batch(&self, item_ids: &[i32]) -> Result<Vec<DofusItem>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = item_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!("{}/items?_id[$in]={}", API_BASE, ids_str);
        self.fetch_list(&url).await
    }

    /// Generic fetch for single item
    async fn fetch_one<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed: {}", response.status());
        }

        response
            .json::<T>()
            .await
            .context("Failed to parse response")
    }

    /// Generic fetch for list of items
    async fn fetch_list<T: DeserializeOwned>(&self, url: &str) -> Result<Vec<T>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed: {}", response.status());
        }

        let data = response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse response")?;
        
        log::debug!("[API] Raw response: {}", serde_json::to_string_pretty(&data).unwrap_or_default());
        
        // DofusDB API returns either array or object with "data" field
        let items = if let Some(array) = data.as_array() {
            log::debug!("[API] Parsing as array with {} items", array.len());
            serde_json::from_value(serde_json::Value::Array(array.clone()))
                .context("Failed to parse items array")?
        } else if let Some(data_field) = data.get("data") {
            log::debug!("[API] Parsing data field");
            serde_json::from_value(data_field.clone())
                .context("Failed to parse data field")?
        } else {
            log::warn!("[API] No data or array found in response");
            Vec::new()
        };

        Ok(items)
    }

    /// Get image URL for an item
    pub fn get_image_url(&self, img_path: &str) -> String {
        if img_path.starts_with("http") {
            img_path.to_string()
        } else {
            format!("https://api.dofusdb.fr{}", img_path)
        }
    }
}

impl Default for DofusApiClient {
    fn default() -> Self {
        Self::new()
    }
}
