use crate::api::request::Model;
use serde::Deserialize;
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl ChatUsage {
    /// Estimated cost in USD for this usage, based on the given model's per-token pricing.
    pub fn cost_usd(&self, model: &Model) -> f64 {
        let pricing = model.pricing();
        let input_cost = self.prompt_tokens as f64 / 1_000_000.0 * pricing.input_per_million_usd;
        let output_cost =
            self.completion_tokens as f64 / 1_000_000.0 * pricing.output_per_million_usd;
        input_cost + output_cost
    }
}

impl std::ops::AddAssign for ChatUsage {
    fn add_assign(&mut self, rhs: Self) {
        self.prompt_tokens += rhs.prompt_tokens;
        self.completion_tokens += rhs.completion_tokens;
        self.total_tokens += rhs.total_tokens;
    }
}

/// Approximate per-million-token pricing in USD, used to estimate request cost.
pub struct ModelPricing {
    pub input_per_million_usd: f64,
    pub output_per_million_usd: f64,
}

impl Model {
    pub fn pricing(&self) -> ModelPricing {
        match self {
            Model::Flash => ModelPricing {
                input_per_million_usd: 0.14,
                output_per_million_usd: 0.28,
            },
            Model::Pro => ModelPricing {
                input_per_million_usd: 0.55,
                output_per_million_usd: 2.19,
            },
        }
    }
}
