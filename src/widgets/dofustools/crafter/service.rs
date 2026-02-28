use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::api_client::DofusApiClient;
use super::types::*;

// Commands sent from UI to worker
#[derive(Debug, Clone)]
enum CrafterCommand {
    SearchItems(String),
    CalculateProfitability(i32),
}

// Results sent from worker to UI
#[derive(Debug, Clone)]
enum CrafterResult {
    SearchResults(Vec<DofusItem>),
    ProfitabilityResult(Result<ProfitabilityResult, String>),
}

pub struct CrafterService {
    state: Arc<RwLock<CrafterState>>,
    data_path: PathBuf,
    command_tx: mpsc::UnboundedSender<CrafterCommand>,
}

impl EventEmitter<CrafterStateChanged> for CrafterService {}
impl EventEmitter<SearchCompleted> for CrafterService {}
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
        let state_clone = Arc::clone(&state);
        let data_path_clone = data_path.clone();

        let (command_tx, command_rx) = mpsc::unbounded::<CrafterCommand>();
        let (result_tx, mut result_rx) = mpsc::unbounded::<CrafterResult>();

        // Worker task (Tokio) - handles API calls
        gpui_tokio::Tokio::spawn(cx, async move {
            Self::crafter_worker(command_rx, result_tx).await
        })
        .detach();

        // UI task (GPUI) - handles results and updates state
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(result) = result_rx.next().await {
                    match result {
                        CrafterResult::SearchResults(items) => {
                            log::info!("[Crafter] Received {} search results in UI task", items.len());
                            let _ = this.update(&mut cx, |_, cx| {
                                cx.emit(SearchCompleted { items });
                                cx.notify();
                            });
                        }
                        CrafterResult::ProfitabilityResult(Ok(profit_result)) => {
                            // Save to history
                            {
                                let mut state = state_clone.write();
                                state.history.push(CalculationHistory {
                                    timestamp: chrono::Utc::now(),
                                    item_id: profit_result.item_id,
                                    item_name: profit_result.item_name.clone(),
                                    craft_cost: profit_result.craft_cost,
                                    market_price: profit_result.market_price,
                                    profit: profit_result.profit,
                                    profit_margin: profit_result.profit_margin,
                                });

                                if state.history.len() > 100 {
                                    state.history.remove(0);
                                }
                            }

                            Self::save_state_static(&data_path_clone, &state_clone);

                            let _ = this.update(&mut cx, |_, cx| {
                                cx.emit(CalculationCompleted {
                                    result: profit_result,
                                });
                                cx.emit(CrafterStateChanged);
                                cx.notify();
                            });
                        }
                        CrafterResult::ProfitabilityResult(Err(_)) => {
                            // Error handled by widget
                        }
                    }
                }
            }
        })
        .detach();

        Self {
            state,
            data_path,
            command_tx,
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

    /// Worker task - runs in Tokio runtime
    async fn crafter_worker(
        mut command_rx: mpsc::UnboundedReceiver<CrafterCommand>,
        result_tx: mpsc::UnboundedSender<CrafterResult>,
    ) {
        let api_client = DofusApiClient::new();

        while let Some(command) = command_rx.next().await {
            match command {
                CrafterCommand::SearchItems(query) => {
                    log::info!("[Crafter] Searching items with query: {}", query);
                    match api_client.search_items(&query, 20).await {
                        Ok(items) => {
                            log::info!("[Crafter] Found {} items", items.len());
                            let _ = result_tx.unbounded_send(CrafterResult::SearchResults(items));
                        }
                        Err(e) => {
                            log::error!("[Crafter] Search error: {}", e);
                        }
                    }
                }
                CrafterCommand::CalculateProfitability(item_id) => {
                    match Self::calculate_profitability_internal(&api_client, item_id).await {
                        Ok(result) => {
                            let _ = result_tx.unbounded_send(
                                CrafterResult::ProfitabilityResult(Ok(result)),
                            );
                        }
                        Err(e) => {
                            let _ = result_tx.unbounded_send(
                                CrafterResult::ProfitabilityResult(Err(e.to_string())),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Internal calculation logic (runs in worker)
    async fn calculate_profitability_internal(
        api_client: &DofusApiClient,
        item_id: i32,
    ) -> Result<ProfitabilityResult> {
        // Get recipe
        let recipe = api_client
            .get_recipe(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No recipe found for item {}", item_id))?;

        // Get result item
        let result_item = api_client.get_item(item_id).await?;
        let item_name = result_item
            .name
            .get("fr")
            .or_else(|| result_item.name.get("en"))
            .cloned()
            .unwrap_or_else(|| format!("Item {}", item_id));

        // Get ingredient items
        let ingredient_items = api_client.get_items_batch(&recipe.ingredient_ids).await?;

        // Build ingredient map
        let mut ingredient_map: HashMap<i32, DofusItem> = HashMap::new();
        for item in ingredient_items {
            ingredient_map.insert(item.id, item);
        }

        // Calculate costs (using default prices for now)
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

            // Default price (will be customizable later)
            let unit_price = 0.0;
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

        // Default market price
        let market_price = 0.0;
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

    /// Load state from disk
    fn load_state(path: &PathBuf) -> CrafterState {
        if let Ok(data) = std::fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            CrafterState::default()
        }
    }

    /// Save state to disk (static version for use in async context)
    fn save_state_static(path: &PathBuf, state: &Arc<RwLock<CrafterState>>) {
        let state_data = state.read().clone();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&state_data) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Save state to disk
    fn save_state(&self) {
        Self::save_state_static(&self.data_path, &self.state)
    }

    /// Get current state
    pub fn get_state(&self) -> CrafterState {
        self.state.read().clone()
    }

    /// Search items (sends command to worker)
    pub fn search_items(&self, query: String) {
        let _ = self.command_tx.unbounded_send(CrafterCommand::SearchItems(query));
    }

    /// Calculate profitability (sends command to worker)
    pub fn calculate_profitability(&self, item_id: i32) {
        let _ = self
            .command_tx
            .unbounded_send(CrafterCommand::CalculateProfitability(item_id));
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
