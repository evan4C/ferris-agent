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
    /// TODO: Consider peak vs offpeak and cached vs uncached pricing.
    pub fn cost_usd(&self, model: &Model) -> f64 {
        let pricing = model.pricing();
        let input_cost = self.prompt_tokens as f64 / 1_000_000.0 * pricing.input_per_million_cached_offpeak_usd;
        let output_cost =
            self.completion_tokens as f64 / 1_000_000.0 * pricing.output_per_million_offpeak_usd;
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
    pub input_per_million_cached_offpeak_usd: f64,
    pub input_per_million_cached_peak_usd: f64,
    pub input_per_million_uncached_offpeak_usd: f64,
    pub input_per_million_uncached_peak_usd: f64,
    pub output_per_million_offpeak_usd: f64,
    pub output_per_million_peak_usd: f64,
}

impl Model {
    pub fn pricing(&self) -> ModelPricing {
        match self {
            // ref: https://api-docs.deepseek.com/quick_start/pricing/
            Model::Flash => ModelPricing {
                input_per_million_cached_offpeak_usd: 0.003,
                input_per_million_cached_peak_usd: 0.006,
                input_per_million_uncached_offpeak_usd: 0.15,
                input_per_million_uncached_peak_usd: 0.3,

                output_per_million_offpeak_usd: 0.6,
                output_per_million_peak_usd: 1.2,
            },
            Model::Pro => ModelPricing {
                input_per_million_cached_offpeak_usd: 0.022,
                input_per_million_cached_peak_usd: 0.044,
                input_per_million_uncached_offpeak_usd: 0.66,
                input_per_million_uncached_peak_usd: 1.32,

                output_per_million_offpeak_usd: 1.98,
                output_per_million_peak_usd: 3.96,
            },
        }
    }
}
