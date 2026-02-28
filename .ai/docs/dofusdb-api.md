# DofusDB API Documentation

Complete reference for the DofusDB API (https://api.dofusdb.fr)

## Table of Contents

- [Overview](#overview)
- [Base Information](#base-information)
- [Response Structure](#response-structure)
- [Endpoints](#endpoints)
- [Query Operators](#query-operators)
- [Common Patterns](#common-patterns)
- [Rust Implementation](#rust-implementation)
- [Examples](#examples)

---

## Overview

The DofusDB API is a **FeathersJS** application providing access to Dofus game data including items, recipes, jobs, monsters, and more.

### Key Characteristics

- **No authentication required**
- **No explicit rate limits** (use responsibly)
- **RESTful JSON API**
- **MongoDB-style query operators**
- **Multi-language support** (en, fr, de, es, pt)

---

## Base Information

| Property | Value |
|----------|-------|
| **Base URL** | `https://api.dofusdb.fr` |
| **Protocol** | HTTPS only |
| **Format** | JSON |
| **Framework** | FeathersJS |

---

## Response Structure

All list endpoints return a paginated response with this structure:

```json
{
  "total": 20583,      // Total number of matching items
  "limit": 10,         // Number of items per page
  "skip": 0,           // Number of items skipped
  "data": [...]        // Array of actual data
}
```

Single item endpoints (e.g., `/items/44`) return the object directly without wrapping.

---

## Endpoints

### Items

**Endpoint**: `/items`

**Total items**: ~20,583

**Query by ID**:
```
GET /items/44
```

**Search items**:
```
GET /items?name.fr[$regex]=.*épée.*&name.fr[$options]=i&$limit=10
```

**Filter by level**:
```
GET /items?level=7
GET /items?level[$gte]=10&level[$lte]=20
```

**Filter by type**:
```
GET /items?typeId=6
```

#### Item Structure

```json
{
  "_id": "674e524064788cc7414184e6",
  "id": 44,
  "typeId": 6,
  "iconId": 6007,
  "level": 7,
  "realWeight": 1,
  "price": 700,
  "name": {
    "de": "Hölzernes Schwert",
    "en": "Twiggy Sword",
    "es": "Espada de maderucha",
    "fr": "Épée de Boisaille",
    "pt": "Espada de graveto"
  },
  "description": {
    "de": "...",
    "en": "...",
    "es": "...",
    "fr": "...",
    "pt": "..."
  },
  "slug": {
    "de": "holzernes schwert",
    "en": "twiggy sword",
    "es": "espada de maderucha",
    "fr": "epee de boisaille",
    "pt": "espada de graveto"
  },
  "img": "https://api.dofusdb.fr/img/items/6007.png",
  "type": { ... },
  "effects": [ ... ],
  "hasRecipe": true,
  "exchangeable": true,
  "className": "WeaponData"
}
```

#### Important Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Unique item ID |
| `name` | object | Localized names (en, fr, de, es, pt) |
| `description` | object | Localized descriptions |
| `slug` | object | URL-friendly names |
| `level` | integer | Required level |
| `typeId` | integer | Item type ID |
| `img` | string | Image URL |
| `hasRecipe` | boolean | Can be crafted |
| `price` | integer | Base NPC price |

---

### Recipes

**Endpoint**: `/recipes`

**Total recipes**: ~4,745

**Query by result ID**:
```
GET /recipes?resultId=44
```

**Get recipe by ID**:
```
GET /recipes/44
```

#### Recipe Structure

```json
{
  "_id": "674e524b64788cc741437078",
  "id": 44,
  "resultId": 44,
  "resultTypeId": 6,
  "resultLevel": 7,
  "ingredientIds": [16512, 303],
  "quantities": [3, 3],
  "jobId": 11,
  "skillId": 20,
  "resultName": {
    "de": "Hölzernes Schwert",
    "en": "Twiggy Sword",
    "fr": "Épée de Boisaille",
    ...
  },
  "result": { ... },      // Full item object
  "ingredients": [ ... ], // Full ingredient objects
  "job": { ... }          // Full job object
}
```

#### Important Fields

| Field | Type | Description |
|-------|------|-------------|
| `resultId` | integer | ID of crafted item |
| `ingredientIds` | array | IDs of required ingredients |
| `quantities` | array | Quantity for each ingredient |
| `jobId` | integer | Required job ID |
| `skillId` | integer | Required skill ID |

---

## Query Operators

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$gt` | Greater than | `level[$gt]=10` |
| `$gte` | Greater than or equal | `level[$gte]=10` |
| `$lt` | Less than | `level[$lt]=50` |
| `$lte` | Less than or equal | `level[$lte]=50` |
| `$ne` | Not equal | `typeId[$ne]=6` |
| `$in` | In array | `_id[$in]=44,49,55` |

### String Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$regex` | Regular expression | `name.fr[$regex]=.*épée.*` |
| `$options` | Regex options (i=case-insensitive) | `name.fr[$options]=i` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$or` | Logical OR | Complex, see examples |
| `$and` | Logical AND | Complex, see examples |

### Pagination

| Parameter | Description | Example |
|-----------|-------------|---------|
| `$limit` | Max results | `$limit=20` |
| `$skip` | Skip results | `$skip=10` |

---

## Common Patterns

### Search by Name (Case-Insensitive)

**❌ WRONG** (returns 400 Bad Request):
```
/items?name[$search]=sword
/items?name[$regex]=sword
```

**✅ CORRECT** (name is an object with language keys):
```
/items?name.fr[$regex]=.*épée.*&name.fr[$options]=i
/items?name.en[$regex]=.*sword.*&name.en[$options]=i
```

### Search Multiple Languages

Use `$or` operator (URL encoding required):
```
/items?$or[0][name.fr][$regex]=.*épée.*&$or[1][name.en][$regex]=.*sword.*
```

### Filter by Level Range

```
/items?level[$gte]=10&level[$lte]=20&$limit=50
```

### Get Items by Multiple IDs

```
/items?_id[$in]=44,49,55,62
```

### Pagination

```
# Page 1 (items 0-19)
/items?$limit=20&$skip=0

# Page 2 (items 20-39)
/items?$limit=20&$skip=20

# Page 3 (items 40-59)
/items?$limit=20&$skip=40
```

---

## Rust Implementation

### Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
urlencoding = "2.1"
```

### API Client

```rust
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

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

    /// Search items by name (French)
    pub async fn search_items(&self, query: &str, limit: usize) -> Result<Vec<DofusItem>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/items?name.fr[$regex]=.*{}.*&name.fr[$options]=i&$limit={}",
            API_BASE, encoded_query, limit
        );
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

        #[derive(Deserialize)]
        struct ApiResponse<T> {
            data: Vec<T>,
        }

        let api_response = response
            .json::<ApiResponse<T>>()
            .await
            .context("Failed to parse response")?;

        Ok(api_response.data)
    }
}
```

### Data Structures

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DofusItem {
    #[serde(rename = "_id")]
    pub id: i32,
    pub name: HashMap<String, String>,
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
```

---

## Examples

### Example 1: Search for Swords

```bash
curl "https://api.dofusdb.fr/items?name.fr[\$regex]=.*épée.*&name.fr[\$options]=i&\$limit=5"
```

### Example 2: Get Item by ID

```bash
curl "https://api.dofusdb.fr/items/44"
```

### Example 3: Get Recipe for Item

```bash
curl "https://api.dofusdb.fr/recipes?resultId=44"
```

### Example 4: Filter by Level Range

```bash
curl "https://api.dofusdb.fr/items?level[\$gte]=10&level[\$lte]=20&\$limit=10"
```

### Example 5: Get Multiple Items

```bash
curl "https://api.dofusdb.fr/items?_id[\$in]=44,49,55"
```

---

## Best Practices

1. **Always URL-encode query parameters** - Especially for regex patterns with special characters
2. **Use pagination** - Don't fetch all items at once, use `$limit` and `$skip`
3. **Cache responses** - API data doesn't change frequently
4. **Handle errors gracefully** - Check HTTP status codes
5. **Use language-specific fields** - Remember that `name` is an object, not a string
6. **Respect the API** - No explicit rate limits, but don't abuse it

---

## Common Pitfalls

### ❌ Searching name directly
```
/items?name[$regex]=sword  // WRONG - returns 400
```

### ✅ Search language-specific field
```
/items?name.en[$regex]=.*sword.*&name.en[$options]=i  // CORRECT
```

### ❌ Forgetting URL encoding
```
/items?name.fr[$regex]=.*épée.*  // May fail with special chars
```

### ✅ URL encode special characters
```
/items?name.fr[$regex]=.*%C3%A9p%C3%A9e.*  // CORRECT
```

---

## Additional Resources

- **DofusDB Website**: https://dofusdb.fr
- **API Base URL**: https://api.dofusdb.fr
- **FeathersJS Docs**: https://feathersjs.com (for query syntax reference)

---

## Changelog

- **2026-02-28**: Initial documentation created
- Tested endpoints: `/items`, `/recipes`
- Confirmed query operators: `$regex`, `$options`, `$in`, `$gte`, `$lte`, `$limit`, `$skip`
- Verified response structure and pagination

---

## License

This documentation is provided as-is for the nwidgets project. The DofusDB API is maintained by the DofusDB team.
