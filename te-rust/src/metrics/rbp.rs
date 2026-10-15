use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct RbpMeasure {
    p: f64,
    _params_str: String,
    sub_metrics: Vec<String>,
}

impl RbpMeasure {
    pub fn new(p: f64, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "rbp".to_string()
        } else {
            format!("rbp_{}", params_str)
        };
        Self {
            p,
            _params_str: params_str.to_string(),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for RbpMeasure {
    fn name(&self) -> &'static str {
        "rbp"
    }

    fn short_description(&self) -> &'static str {
        "Rank-Biased Precision"
    }

    fn explanation(&self) -> &'static str {
        "Rank-Biased Precision. RBP measures the rate at which utility is gained by a user working at a given degree of persistence p. By default, p = 0.9. Higher values of p simulate a more persistent user who reads deeper into the ranking list."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let default_gains_config = GainsConfig::parse("");
        let gains_config = if let Some(ref gg) = config.global_gains {
            gg
        } else {
            &default_gains_config
        };

        let gains = Gains::setup(gains_config, &q_state.rel_levels);

        // Find min and max gains to normalize if needed
        let mut min_gain = f64::INFINITY;
        let mut max_gain = f64::NEG_INFINITY;
        for g in &gains.rel_gains {
            if g.gain < min_gain {
                min_gain = g.gain;
            }
            if g.gain > max_gain {
                max_gain = g.gain;
            }
        }

        let get_normalized_gain = |rel_level: i64| -> f64 {
            let raw_gain = gains.get_gain(rel_level);
            if min_gain < 0.0 || max_gain > 1.0 {
                let range = max_gain - min_gain;
                if range > 0.0 {
                    (raw_gain - min_gain) / range
                } else {
                    0.0
                }
            } else {
                raw_gain
            }
        };

        let mut sum = 0.0;
        let mut cur_p = 1.0;

        for &rel in &q_state.results_rel_list {
            let gain = get_normalized_gain(rel);
            if gain != 0.0 {
                sum += gain * cur_p;
            }
            cur_p *= self.p;
        }

        let val = (1.0 - self.p) * sum;
        vec![MetricValue::Float(val)]
    }
}

impl Default for RbpMeasure {
    fn default() -> Self {
        Self::new(0.9, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rbp_standard() {
        let measure = RbpMeasure::new(0.9, "");
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. relevance level = 1.
        // gains at rel levels: rel=0 -> gain=0, rel=1 -> gain=1.
        // normalized gains: max=1, min=0 (standard, no change).
        // rank 1: rel=1 -> gain=1. sum += 1 * p^0 = 1.0
        // rank 2: rel=0 -> gain=0. sum += 0
        // rank 3: rel=1 -> gain=1. sum += 1 * p^2 = 0.81
        // total sum = 1.81.
        // result = (1 - 0.9) * 1.81 = 0.1 * 1.81 = 0.181
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.181).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_rbp_custom_p() {
        let measure = RbpMeasure::new(0.5, "p=0.5");
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2.
        // sum = 1 * 0.5^0 + 0 + 1 * 0.5^2 = 1.0 + 0.25 = 1.25.
        // result = (1 - 0.5) * 1.25 = 0.625
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.625)]);
    }
}
