use anyhow::Result;
use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::api_client::DofusApiClient;
use super::types::*;

pub struct CrafterService {
    state: Arc<RwLock<CrafterState>>,
    api_client: Arc<DofusApiClient>,
    data_path: PathBuf,
}

impl EventEmitter<CrafterStateChanged> for CrafterService {}
impl EventEmitter<CalculationCompleted> for CrafterService {}

struct GlobalCrafterService(Entity<CrafterService>);
impl Global for GlobalCrafterService {}

impl CrafterService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let data_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nwidgets")
            .join("crafter_data.json");

        let state = Arc::new(RwLock::new(Self::load_state(&data_path)));
        let api_client = Arc::new(DofusApiClient::new());

        Self {
            state,
            api_client,
            data_path,
        }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCrafterService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalCrafterService(service.clone()));
        service
    }

    /// Load state from disk
    fn load_state(path: &PathBuf) -> CrafterState {
        if let Ok(data) = std::fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            CrafterState::default()
        }
    }

    /// Save state to disk
    fn save_state(&self) {
        let state = self.state.read().clone();
        if let Some(parent) = self.data_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(&self.data_path, json);
        }
    }

    /// Get current state
    pub fn get_state(&self) -> CrafterState {
        self.state.read().clone()
    }

    /// Search items by name
    pub async fn search_items(&self, query: String) -> Result<Vec<DofusItem>> {
        self.api_client.search_items(&query, 20).await
    }

    /// Get item details
    pub async fn get_item(&self, item_id: i32) -> Result<DofusItem> {
        self.api_client.get_item(item_id).await
    }

    /// Calculate profitability for an item
    pub async fn calculate_profitability(
        &self,
        item_id: i32,
    ) -> Result<ProfitabilityResult> {
        // Get recipe
        let recipe = self
            .api_client
            .get_recipe(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No recipe found for item {}", item_id))?;

        // Get result item
        let result_item = self.api_client.get_item(item_id).await?;
        let item_name = result_item
            .name
            .get("fr")
            .or_else(|| result_item.name.get("en"))
            .cloned()
            .unwrap_or_else(|| format!("Item {}", item_id));

        // Get ingredient items
        let ingredient_items = self
            .api_client
            .get_items_batch(&recipe.ingredient_ids)
            .await?;

        // Build ingredient map
        let mut ingredient_map: HashMap<i32, DofusItem> = HashMap::new();
        for item in ingredient_items {
            ingredient_map.insert(item.id, item);
        }

        // Calculate costs
        let state = self.state.read();
        let mut ingredients = Vec::new();
        let mut total_craft_cost = 0.0;

        for (idx, &ingredient_id) in recipe.ingredient_ids.iter().enumerate() {
            let quantity = recipe.quantities.get(idx).copied().unwrap_or(1);
            let ingredient_item = ingredient_map.get(&ingredient_id);

            let ingredient_name = ingredient_item
                .and_then(|item| {
                    item.name
                        .get("fr")
                        .or_else(|| item.name.get("en"))
                        .cloned()
                })
                .unwrap_or_else(|| format!("Item {}", ingredient_id));

            // Get custom price or default to 0
            let unit_price = state
                .pricings
                .get(&item_id)
                .and_then(|p| p.resource_prices.get(&ingredient_id).copied())
                .unwrap_or(0.0);

            let total_cost = unit_price * quantity as f64;
            total_craft_cost += total_cost;

            ingredients.push(IngredientCost {
                item_id: ingredient_id,
                item_name: ingredient_name,
                quantity,
                unit_price,
                total_cost,
            });
        }

        // Get market price
        let market_price = state
            .pricings
            .get(&item_id)
            .and_then(|p| p.market_price)
            .unwrap_or(0.0);

        let profit = market_price - total_craft_cost;
        let profit_margin = if market_price > 0.0 {
            (profit / market_price) * 100.0
        } else {
            0.0
        };

        Ok(ProfitabilityResult {
            item_id,
            item_name,
            craft_cost: total_craft_cost,
            market_price,
            profit,
            profit_margin,
            is_profitable: profit > 0.0,
            recipe,
            ingredients,
        })
    }

    /// Set market price for an item
    pub fn set_market_price(&self, item_id: i32, item_name: String, price: f64, cx: &mut Context<Self>) {
        let mut state = self.state.write();
        let pricing = state.pricings.entry(item_id).or_insert_with(|| ItemPricing {
            item_id,
            item_name: item_name.clone(),
            craft_cost: None,
            market_price: None,
            resource_prices: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });

        pricing.market_price = Some(price);
        pricing.last_updated = chrono::Utc::now();
        drop(state);

        self.save_state();
        cx.emit(CrafterStateChanged);
        cx.notify();
    }

    /// Set resource price
    pub fn set_resource_price(
        &self,
        item_id: i32,
        item_name: String,
        resource_id: i32,
        price: f64,
        cx: &mut Context<Self>,
    ) {
        let mut state = self.state.write();
        let pricing = state.pricings.entry(item_id).or_insert_with(|| ItemPricing {
            item_id,
            item_name: item_name.clone(),
            craft_cost: None,
            market_price: None,
            resource_prices: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });

        pricing.resource_prices.insert(resource_id, price);
        pricing.last_updated = chrono::Utc::now();
        drop(state);

        self.save_state();
        cx.emit(CrafterStateChanged);
        cx.notify();
    }

    /// Add calculation to history
    pub fn add_to_history(&self, result: &ProfitabilityResult, cx: &mut Context<Self>) {
        let mut state = self.state.write();
        state.history.push(CalculationHistory {
            timestamp: chrono::Utc::now(),
            item_id: result.item_id,
            item_name: result.item_name.clone(),
            craft_cost: result.craft_cost,
            market_price: result.market_price,
            profit: result.profit,
            profit_margin: result.profit_margin,
        });

        // Keep only last 100 entries
        if state.history.len() > 100 {
            state.history.remove(0);
        }

        drop(state);
        self.save_state();
        cx.emit(CrafterStateChanged);
        cx.notify();
    }

    /// Toggle favorite
    pub fn toggle_favorite(&self, item_id: i32, cx: &mut Context<Self>) {
        let mut state = self.state.write();
        if let Some(pos) = state.favorites.iter().position(|&id| id == item_id) {
            state.favorites.remove(pos);
        } else {
            state.favorites.push(item_id);
        }
        drop(state);

        self.save_state();
        cx.emit(CrafterStateChanged);
        cx.notify();
    }

    /// Clear history
    pub fn clear_history(&self, cx: &mut Context<Self>) {
        let mut state = self.state.write();
        state.history.clear();
        drop(state);

        self.save_state();
        cx.emit(CrafterStateChanged);
        cx.notify();
    }
}
