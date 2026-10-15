use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct NdcgMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgMeasure {
    pub fn new(params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "ndcg".to_string()
        } else {
            format!("ndcg_{}", params_str)
        };
        Self {
            gains_config: GainsConfig::parse(params_str),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for NdcgMeasure {
    fn name(&self) -> &'static str {
        "ndcg"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain. Computes NDCG over the entire retrieved ranking and entire ideal ranking."
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
        let mut ideal_dcg = 0.0;
        let mut cur_level = gains.rel_gains.len() as i64 - 1;
        let mut ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
        let mut num_at_level = 0;

        let mut i = 0;
        while i < q_state.num_ret && ideal_gain > 0.0 {
            let results_gain = gains.get_gain(q_state.results_rel_list[i]);
            if results_gain != 0.0 {
                results_dcg += results_gain / ((i + 2) as f64).log2();
            }

            num_at_level += 1;
            while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                num_at_level = 1;
                cur_level -= 1;
                ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
            }

            if ideal_gain > 0.0 {
                ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
            }

            i += 1;
        }

        while i < q_state.num_ret {
            let results_gain = gains.get_gain(q_state.results_rel_list[i]);
            if results_gain != 0.0 {
                results_dcg += results_gain / ((i + 2) as f64).log2();
            }
            i += 1;
        }

        while ideal_gain > 0.0 {
            num_at_level += 1;
            while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                num_at_level = 1;
                cur_level -= 1;
                ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
            }
            if ideal_gain > 0.0 {
                ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
            }
            i += 1;
        }

        let score = if ideal_dcg > 0.0 { results_dcg / ideal_dcg } else { 0.0 };
        vec![MetricValue::Float(score)]
    }

}

impl Default for NdcgMeasure {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_standard() {
        let measure = NdcgMeasure::new("");
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // dcg = 1.0 / log2(2) + 0.0 + 1.0 / log2(4) = 1.0 + 0.5 = 1.5.
        // idcg = 1.0 / log2(2) + 1.0 / log2(3) = 1.0 + 1.0 / 1.58496 = 1.63092975
        // ndcg = 1.5 / 1.63092975 = 0.91972079
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.919720791448).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
