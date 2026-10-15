use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct NdcgPMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgPMeasure {
    pub fn new(params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "ndcg_p".to_string()
        } else {
            format!("ndcg_p_{}", params_str)
        };
        Self {
            gains_config: GainsConfig::parse(params_str),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for NdcgPMeasure {
    fn name(&self) -> &'static str {
        "ndcg_p"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain (parameter based)"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain (parameter based). Compute traditional NDCG with base 2 logarithmic discount where rank 1 is not discounted, and rank i+1 is divided by log2(i+1)."
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

        let gains_config = if !self.gains_config.custom_gains.is_empty() {
            &self.gains_config
        } else if let Some(ref gg) = config.global_gains {
            gg
        } else {
            &self.gains_config
        };
        let gains = Gains::setup(gains_config, &q_state.rel_levels);

        let mut results_dcg = 0.0;
        for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
            let gain = gains.get_gain(rel);
            if gain != 0.0 {
                if i == 0 {
                    results_dcg += gain;
                } else {
                    results_dcg += gain / ((i + 1) as f64).log2();
                }
            }
        }

        let mut ideal_dcg = 0.0;
        let mut cur_level = gains.rel_gains.len() as i64 - 1;
        let mut num_at_level = 0;

        for i in 0..gains._total_num_at_levels {
            num_at_level += 1;
            while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                num_at_level = 1;
                cur_level -= 1;
            }

            if cur_level < 0 || gains.rel_gains[cur_level as usize].gain <= 0.0 {
                break;
            }

            let gain = gains.rel_gains[cur_level as usize].gain;
            if i == 0 {
                ideal_dcg += gain;
            } else {
                ideal_dcg += gain / ((i + 1) as f64).log2();
            }
        }

        let score = if q_state.num_rel_ret > 0 && ideal_dcg > 0.0 {
            results_dcg / ideal_dcg
        } else {
            0.0
        };

        vec![MetricValue::Float(score)]
    }

}

impl Default for NdcgPMeasure {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_p_standard() {
        let measure = NdcgPMeasure::new("");
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2.
        // rank 1 (i=0): rel=1 -> gain=1.0. dcg += 1.0 = 1.0
        // rank 2 (i=1): rel=0 -> gain=0.
        // rank 3 (i=2): rel=1 -> gain=1.0. dcg += 1.0 / log2(3) = 1.0 / 1.58496 = 0.63092975
        // total results_dcg = 1.63092975
        // ideal_dcg: i=0 -> gain=1.0. i=1 -> gain=1.0 / log2(2) = 1.0. total ideal_dcg = 2.0
        // ndcg_p = 1.63092975 / 2.0 = 0.81546487
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.81546487).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
