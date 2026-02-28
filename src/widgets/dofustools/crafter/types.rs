use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Item data from DofusDB API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DofusItem {
    #[serde(rename = "_id")]
    pub _id: String,
    pub id: i32,
    pub name: HashMap<String, String>, // Localized names
    pub level: i32,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub img: Option<String>,
    pub description: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemType {
    pub id: i32,
    pub name: HashMap<String, String>,
}

/// Recipe data from DofusDB API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DofusRecipe {
    #[serde(rename = "_id")]
    pub id: i32,
    #[serde(rename = "resultId")]
    pub result_id: i32,
    #[serde(rename = "ingredientIds")]
    pub ingredient_ids: Vec<i32>,
    pub quantities: Vec<i32>,
    #[serde(rename = "jobId")]
    pub job_id: Option<i32>,
}

/// User-defined pricing for an item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPricing {
    pub item_id: i32,
    pub item_name: String,
    pub craft_cost: Option<f64>,      // Total cost to craft
    pub market_price: Option<f64>,    // HDV (auction house) price
    pub resource_prices: HashMap<i32, f64>, // Custom prices for each resource
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Profitability calculation result
#[derive(Debug, Clone)]
pub struct ProfitabilityResult {
    pub item_id: i32,
    pub item_name: String,
    pub craft_cost: f64,
    pub market_price: f64,
    pub profit: f64,
    pub profit_margin: f64, // Percentage
    pub is_profitable: bool,
    pub recipe: DofusRecipe,
    pub ingredients: Vec<IngredientCost>,
}

#[derive(Debug, Clone)]
pub struct IngredientCost {
    pub item_id: i32,
    pub item_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_cost: f64,
}

/// History entry for tracking calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationHistory {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub item_id: i32,
    pub item_name: String,
    pub craft_cost: f64,
    pub market_price: f64,
    pub profit: f64,
    pub profit_margin: f64,
}

/// Crafter service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrafterState {
    pub pricings: HashMap<i32, ItemPricing>,
    pub history: Vec<CalculationHistory>,
    pub favorites: Vec<i32>, // Favorite item IDs
}

impl Default for CrafterState {
    fn default() -> Self {
        Self {
            pricings: HashMap::new(),
            history: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

/// Events emitted by CrafterService
#[derive(Debug, Clone)]
pub struct CrafterStateChanged;

#[derive(Debug, Clone)]
pub struct CalculationCompleted {
    pub result: ProfitabilityResult,
}

#[derive(Debug, Clone)]
pub struct SearchCompleted {
    pub items: Vec<DofusItem>,
}
