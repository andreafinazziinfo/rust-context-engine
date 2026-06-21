use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const PRICING_JSON: &str = include_str!("../../../data/model_pricing.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPrice {
    pub model_id: String,
    pub provider: String,
    pub display_name: String,
    pub input_price_per_mtok: f64,
    pub output_price_per_mtok: f64,
    pub last_verified: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRegistry {
    pub pricing_revision: String,
    pub source: String,
    pub tokenizer_note: String,
    pub models: Vec<ModelPrice>,
}

static REGISTRY: LazyLock<PricingRegistry> = LazyLock::new(|| {
    serde_json::from_str(PRICING_JSON).expect("failed to parse embedded model_pricing.json")
});

/// Returns the whole registry
pub fn get_registry() -> &'static PricingRegistry {
    &REGISTRY
}

/// Retrieve the price entry for a model by ID (e.g. "claude-3.5-sonnet").
/// Supports partial/case-insensitive matches.
pub fn get_model_price(model_id: &str) -> Option<&'static ModelPrice> {
    let target = model_id.to_lowercase();
    REGISTRY.models.iter().find(|m| {
        m.model_id.to_lowercase() == target
            || target.contains(&m.model_id.to_lowercase())
            || m.model_id.to_lowercase().contains(&target)
    })
}

/// Calculate cost for a given number of tokens and model.
/// If model is not found, defaults to $3.00 per MTok (input) / $15.00 per MTok (output) (Claude 3.5 Sonnet base).
pub fn calculate_cost(tokens: i64, model_id: &str, is_output: bool) -> f64 {
    let price_per_mtok = match get_model_price(model_id) {
        Some(m) => {
            if is_output {
                m.output_price_per_mtok
            } else {
                m.input_price_per_mtok
            }
        }
        None => {
            if is_output {
                15.00
            } else {
                3.00
            }
        }
    };
    (tokens as f64) * price_per_mtok / 1_000_000.0
}

/// Calculate savings in USD based on tokens saved.
/// Assumes saved tokens are input tokens (since they are pruned from the prompt input).
pub fn calculate_savings(tokens_saved: i64, model_id: &str) -> f64 {
    calculate_cost(tokens_saved, model_id, false)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BudgetStatus {
    pub limit_usd: f64,
    pub spent_usd: f64,
    pub exceeded: bool,
    pub percentage: f64,
}

pub fn check_budget(limit_usd: f64) -> anyhow::Result<BudgetStatus> {
    let spent_usd = crate::tracking::get_total_cost_spent()?;
    let exceeded = spent_usd >= limit_usd;
    let percentage = if limit_usd > 0.0 {
        (spent_usd / limit_usd) * 100.0
    } else {
        0.0
    };
    Ok(BudgetStatus {
        limit_usd,
        spent_usd,
        exceeded,
        percentage,
    })
}

pub fn suggest_model(task_type: &str) -> &'static str {
    match task_type.to_lowercase().as_str() {
        "simple" | "single-file-edit" | "documentation" => "gemini-3.5-flash",
        "complex" | "complex-refactoring" | "planning" => "claude-4.6-sonnet",
        "audit" | "security" | "review" => "gemini-3.1-pro-preview",
        _ => "claude-4.6-sonnet",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_registry_loads() {
        let registry = get_registry();
        assert!(!registry.models.is_empty());
        assert_eq!(registry.pricing_revision, "2026-06-21");
    }

    #[test]
    fn test_get_model_price() {
        let price = get_model_price("claude-4.6-sonnet").unwrap();
        assert_eq!(price.display_name, "Claude Sonnet 4.6");
        assert_eq!(price.input_price_per_mtok, 3.0);
        assert_eq!(price.output_price_per_mtok, 15.0);
    }

    #[test]
    fn test_calculate_cost() {
        // 100,000 input tokens for claude-4.6-sonnet -> $3.00 * 0.1 = $0.30
        let cost = calculate_cost(100_000, "claude-4.6-sonnet", false);
        assert!((cost - 0.30).abs() < 1e-6);

        // 10,000 output tokens for claude-4.6-sonnet -> $15.00 * 0.01 = $0.15
        let cost_out = calculate_cost(10_000, "claude-4.6-sonnet", true);
        assert!((cost_out - 0.15).abs() < 1e-6);
    }
}
