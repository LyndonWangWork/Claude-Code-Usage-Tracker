//! Pricing calculation for Claude models

use std::collections::HashMap;

/// Pricing per million tokens (USD)
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_creation: f64,
    pub cache_read: f64,
}

impl ModelPricing {
    pub fn new(input: f64, output: f64, cache_creation: f64, cache_read: f64) -> Self {
        Self {
            input,
            output,
            cache_creation,
            cache_read,
        }
    }
}

/// Calculator for API costs based on token usage
pub struct PricingCalculator {
    pricing: HashMap<String, ModelPricing>,
    default_pricing: ModelPricing,
}

impl Default for PricingCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingCalculator {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();

        // Opus pricing
        let opus = ModelPricing::new(15.0, 75.0, 18.75, 1.5);
        pricing.insert("claude-3-opus".to_string(), opus.clone());
        pricing.insert("claude-opus-4".to_string(), opus.clone());

        // Sonnet pricing (default)
        let sonnet = ModelPricing::new(3.0, 15.0, 3.75, 0.3);
        pricing.insert("claude-3-sonnet".to_string(), sonnet.clone());
        pricing.insert("claude-3-5-sonnet".to_string(), sonnet.clone());
        pricing.insert("claude-sonnet-4".to_string(), sonnet.clone());

        // Haiku pricing
        let haiku = ModelPricing::new(0.25, 1.25, 0.3, 0.03);
        pricing.insert("claude-3-haiku".to_string(), haiku.clone());
        pricing.insert("claude-3-5-haiku".to_string(), haiku);

        Self {
            pricing,
            default_pricing: sonnet, // Default to Sonnet pricing
        }
    }

    /// Normalize model name for pricing lookup
    fn normalize_model_name(&self, model: &str) -> String {
        let model_lower = model.to_lowercase();

        // Handle Claude 4 models
        if model_lower.contains("opus-4") || model_lower.contains("claude-opus-4") {
            return "claude-opus-4".to_string();
        }
        if model_lower.contains("sonnet-4") || model_lower.contains("claude-sonnet-4") {
            return "claude-sonnet-4".to_string();
        }

        // Handle Claude 3.x models
        if model_lower.contains("opus") {
            return "claude-3-opus".to_string();
        }
        if model_lower.contains("haiku") {
            if model_lower.contains("3.5") || model_lower.contains("3-5") {
                return "claude-3-5-haiku".to_string();
            }
            return "claude-3-haiku".to_string();
        }
        if model_lower.contains("sonnet") {
            if model_lower.contains("3.5") || model_lower.contains("3-5") {
                return "claude-3-5-sonnet".to_string();
            }
            return "claude-3-sonnet".to_string();
        }

        // Default
        "claude-3-5-sonnet".to_string()
    }

    /// Get pricing for a model
    fn get_pricing(&self, model: &str) -> &ModelPricing {
        let normalized = self.normalize_model_name(model);
        self.pricing.get(&normalized).unwrap_or(&self.default_pricing)
    }

    /// Calculate cost for token usage
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
    ) -> f64 {
        let pricing = self.get_pricing(model);

        let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;
        let cache_creation_cost =
            (cache_creation_tokens as f64 / 1_000_000.0) * pricing.cache_creation;
        let cache_read_cost = (cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read;

        // Round to 6 decimal places
        ((input_cost + output_cost + cache_creation_cost + cache_read_cost) * 1_000_000.0).round()
            / 1_000_000.0
    }
}

/// Plan limits
#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub token_limit: u64,
    pub cost_limit: f64,
    pub message_limit: u32,
}

/// Get plan limits by plan type
pub fn get_plan_limits(plan_type: &str) -> PlanLimits {
    match plan_type.to_lowercase().as_str() {
        "pro" => PlanLimits {
            token_limit: 19_000,
            cost_limit: 18.0,
            message_limit: 250,
        },
        "max5" => PlanLimits {
            token_limit: 88_000,
            cost_limit: 35.0,
            message_limit: 1_000,
        },
        "max20" => PlanLimits {
            token_limit: 220_000,
            cost_limit: 140.0,
            message_limit: 2_000,
        },
        _ => PlanLimits {
            token_limit: 19_000,
            cost_limit: 18.0,
            message_limit: 250,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost_sonnet() {
        let calculator = PricingCalculator::new();
        let cost = calculator.calculate_cost("claude-3-5-sonnet", 1_000_000, 1_000_000, 0, 0);
        // Expected: 3.0 + 15.0 = 18.0
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_model_name() {
        let calculator = PricingCalculator::new();
        assert_eq!(
            calculator.normalize_model_name("claude-3-5-sonnet-20240620"),
            "claude-3-5-sonnet"
        );
        assert_eq!(
            calculator.normalize_model_name("Claude 3 Opus"),
            "claude-3-opus"
        );
    }
}
