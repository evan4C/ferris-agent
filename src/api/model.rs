use serde::Serialize;

#[derive(Clone, Debug)]
pub enum Model {
    Flash,
    Pro,
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

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let model = match self {
            Self::Flash => "deepseek-flash",
            Self::Pro => "deepseek-v4-pro",
        };
        serializer.serialize_str(model)
    }
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