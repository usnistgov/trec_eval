use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct GMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl GMeasure {
    pub fn new(params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "G".to_string()
        } else {
            format!("G_{}", params_str)
        };
        Self {
            gains_config: GainsConfig::parse(params_str),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for GMeasure {
    fn name(&self) -> &'static str {
        "G"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Gain"
    }

    fn explanation(&self) -> &'static str {
        "Experimental measure G, combining qualities of MAP and NDCG. G(doc) = gain(doc) / log2(2 + ideal_gain(i) - results_gain(i))."
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


                let mut results_g = 0.0;
                let mut sum_results = 0.0;
                let mut sum_ideal = 0.0;
                let mut sum_cost = 0.0;
                let mut cur_level = gains.rel_gains.len() as i64 - 1;
                let mut ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
                let mut num_at_level = 0;
                let min_cost = 1.0;

                let mut i = 0;
                while i < q_state.num_ret && ideal_gain > 0.0 {
                    let results_gain = gains.get_gain(q_state.results_rel_list[i]);
                    sum_results += results_gain;

                    num_at_level += 1;
                    while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                        num_at_level = 1;
                        cur_level -= 1;
                        ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
                    }

                    if ideal_gain > 0.0 {
                        sum_ideal += ideal_gain;
                    }
                    if ideal_gain >= min_cost {
                        sum_cost += ideal_gain;
                    } else {
                        sum_cost += min_cost;
                    }

                    if results_gain != 0.0 {
                        results_g += results_gain / (2.0 + sum_cost - sum_results).log2();
                    }

                    i += 1;
                }

                while i < q_state.num_ret {
                    let results_gain = gains.get_gain(q_state.results_rel_list[i]);
                    sum_results += results_gain;
                    sum_cost += min_cost;

                    if results_gain != 0.0 {
                        results_g += results_gain / (2.0 + sum_cost - sum_results).log2();
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
                        sum_ideal += ideal_gain;
                    }
                }

                let score = if sum_ideal > 0.0 {
                    results_g / sum_ideal
                } else {
                    0.0
                };

                vec![MetricValue::Float(score)]
    }
}


impl Default for GMeasure {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_g_standard() {
        let measure = GMeasure::new("");
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
    }
}
