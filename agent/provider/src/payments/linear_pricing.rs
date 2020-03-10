use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use serde_json::{Value, json};

use ya_agent_offer_model::ComInfo;

use super::model::{PaymentModel, PaymentDescription};


/// Computes computations costs.
pub struct LinearPricing {
    usage_coeffs: Vec<f64>,
}

impl PaymentModel for LinearPricing {
    fn compute_cost(&self, usage: &Vec<f64>) -> Result<BigDecimal> {
        let cost: f64 = usage.iter()
            .zip(self.usage_coeffs.iter())
            .map(|(coeff, usage_value)| coeff * usage_value)
            .sum();
        Ok(BigDecimal::from(cost))
    }
}

impl LinearPricing {
    pub fn new(commercials: PaymentDescription) -> Result<LinearPricing> {
        Ok(LinearPricing{usage_coeffs: commercials.usage_coeffs})
    }
}

/// Helper for building offer.
pub struct LinearPricingOffer {
    usage_coeffs: Vec<f64>,
    usage_params: Vec<String>,
    interval: f64,
}

impl LinearPricingOffer {
    pub fn new() -> LinearPricingOffer {
        // Initialize first constant coefficient to 0.
        LinearPricingOffer{usage_coeffs: vec![0.0], usage_params: vec![], interval: 60.0}
    }

    pub fn add_coefficient(&mut self, coeff_name: &str, value: f64) -> &mut LinearPricingOffer {
        self.usage_params.push(coeff_name.to_string());
        self.usage_coeffs.push(value);
        return self
    }

    /// Adds constant cost paid no matter how many resources computations will consume.
    pub fn initial_cost(&mut self, value: f64) -> &mut LinearPricingOffer {
        self.usage_coeffs[0] = value;
        return self
    }

    pub fn interval(&mut self, seconds: f64) -> &mut LinearPricingOffer {
        self.interval = seconds;
        return self
    }

    pub fn build(&self) -> ComInfo {
        let params = json!({
            "scheme": "payu".to_string(),
            "scheme.payu": json!({
                "interval_sec": self.interval
            }),
            "pricing": json!({
                "model": "linear".to_string(),
                "model.linear": json!({
                    "coeffs": self.usage_coeffs.clone()
                })
            }),
            "usage": json!({
                "vector": self.usage_params.clone()
            })
        });

        ComInfo{params}
    }
}



