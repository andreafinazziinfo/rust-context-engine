use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const PRICING_JSON: &str = include_str!("../model_pricing.json");

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

fn find_local_pricing_file() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let candidate = current.join(".rtk_pricing.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn get_global_pricing_file() -> Option<std::path::PathBuf> {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };
    home.map(|h| {
        std::path::PathBuf::from(h)
            .join(".config")
            .join("rtk")
            .join("pricing.json")
    })
}

/// Retrieves the pricing for a model, merging any local/global overrides.
pub fn get_merged_price(model_id: &str) -> Option<ModelPrice> {
    let target = model_id.to_lowercase();

    // 1. Try local project override (.rtk_pricing.json)
    if let Some(local_path) = find_local_pricing_file() {
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            if let Ok(override_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(models) = override_data.get("models").and_then(|m| m.as_array()) {
                    for m_val in models {
                        if let Some(mid) = m_val.get("model_id").and_then(|id| id.as_str()) {
                            let mid_lower = mid.to_lowercase();
                            if mid_lower == target
                                || target.contains(&mid_lower)
                                || mid_lower.contains(&target)
                            {
                                let base = get_model_price(&mid_lower);
                                return Some(ModelPrice {
                                    model_id: mid.to_string(),
                                    provider: m_val
                                        .get("provider")
                                        .and_then(|p| p.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.provider.clone()).unwrap_or_default()
                                        }),
                                    display_name: m_val
                                        .get("display_name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.display_name.clone()).unwrap_or_default()
                                        }),
                                    input_price_per_mtok: m_val
                                        .get("input_price_per_mtok")
                                        .and_then(|p| p.as_f64())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.input_price_per_mtok).unwrap_or(3.0)
                                        }),
                                    output_price_per_mtok: m_val
                                        .get("output_price_per_mtok")
                                        .and_then(|p| p.as_f64())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.output_price_per_mtok).unwrap_or(15.0)
                                        }),
                                    last_verified: m_val
                                        .get("last_verified")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                    source_url: m_val
                                        .get("source_url")
                                        .and_then(|u| u.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Try global override (~/.config/rtk/pricing.json)
    if let Some(global_path) = get_global_pricing_file() {
        if let Ok(content) = std::fs::read_to_string(&global_path) {
            if let Ok(override_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(models) = override_data.get("models").and_then(|m| m.as_array()) {
                    for m_val in models {
                        if let Some(mid) = m_val.get("model_id").and_then(|id| id.as_str()) {
                            let mid_lower = mid.to_lowercase();
                            if mid_lower == target
                                || target.contains(&mid_lower)
                                || mid_lower.contains(&target)
                            {
                                let base = get_model_price(&mid_lower);
                                return Some(ModelPrice {
                                    model_id: mid.to_string(),
                                    provider: m_val
                                        .get("provider")
                                        .and_then(|p| p.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.provider.clone()).unwrap_or_default()
                                        }),
                                    display_name: m_val
                                        .get("display_name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.display_name.clone()).unwrap_or_default()
                                        }),
                                    input_price_per_mtok: m_val
                                        .get("input_price_per_mtok")
                                        .and_then(|p| p.as_f64())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.input_price_per_mtok).unwrap_or(3.0)
                                        }),
                                    output_price_per_mtok: m_val
                                        .get("output_price_per_mtok")
                                        .and_then(|p| p.as_f64())
                                        .unwrap_or_else(|| {
                                            base.map(|b| b.output_price_per_mtok).unwrap_or(15.0)
                                        }),
                                    last_verified: m_val
                                        .get("last_verified")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                    source_url: m_val
                                        .get("source_url")
                                        .and_then(|u| u.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback to standard registry
    get_model_price(model_id).cloned()
}

/// Calculate cost for a given number of tokens and model.
/// If model is not found, defaults to $3.00 per MTok (input) / $15.00 per MTok (output) (Claude 3.5 Sonnet base).
pub fn calculate_cost(tokens: i64, model_id: &str, is_output: bool) -> f64 {
    let price_per_mtok = match get_merged_price(model_id) {
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

    #[test]
    fn test_pricing_overrides() {
        let tmp = std::env::temp_dir().join(format!("rtk_pricing_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let tmp_cleanup = tmp.clone();
        struct RestoreCwd {
            cwd: std::path::PathBuf,
            tmp: std::path::PathBuf,
        }
        impl Drop for RestoreCwd {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.cwd);
                let _ = std::fs::remove_dir_all(&self.tmp);
            }
        }
        let _restore = RestoreCwd {
            cwd: original_cwd,
            tmp: tmp_cleanup,
        };

        let override_json = serde_json::json!({
            "models": [
                {
                    "model_id": "claude-4.6-sonnet",
                    "input_price_per_mtok": 1.23,
                    "output_price_per_mtok": 4.56
                }
            ]
        });
        std::fs::write(".rtk_pricing.json", override_json.to_string()).unwrap();

        let price = get_merged_price("claude-4.6-sonnet").unwrap();
        assert_eq!(price.input_price_per_mtok, 1.23);
        assert_eq!(price.output_price_per_mtok, 4.56);
    }
}
