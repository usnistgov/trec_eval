use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct NdcgRelMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgRelMeasure {
    pub fn new(params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "ndcg_rel".to_string()
        } else {
            format!("ndcg_rel_{}", params_str)
        };
        Self {
            gains_config: GainsConfig::parse(params_str),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for NdcgRelMeasure {
    fn name(&self) -> &'static str {
        "ndcg_rel"
    }

    fn short_description(&self) -> &'static str {
        "NDCG relative to rel docs"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain averaged over rel docs. This version averages ndcg over each relevant doc, where relevant is defined as expected gain > 0. If a rel doc is not retrieved, then ndcg for the doc is the dcg at the end of the retrieval / ideal dcg."
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

                let mut sum = 0.0;
                let mut num_rel_ret = 0;
                let mut num_rel = 0;

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
                        num_rel += 1;
                        ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
                    }

                    if results_gain > 0.0 {
                        if ideal_dcg > 0.0 {
                            sum += results_dcg / ideal_dcg;
                        }
                        num_rel_ret += 1;
                    }

                    i += 1;
                }

                while i < q_state.num_ret {
                    let results_gain = gains.get_gain(q_state.results_rel_list[i]);
                    if results_gain != 0.0 {
                        results_dcg += results_gain / ((i + 2) as f64).log2();
                    }

                    if results_gain > 0.0 {
                        if ideal_dcg > 0.0 {
                            sum += results_dcg / ideal_dcg;
                        }
                        num_rel_ret += 1;
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
                        num_rel += 1;
                        ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
                    }
                    i += 1;
                }

                if num_rel > num_rel_ret && ideal_dcg > 0.0 {
                    sum += ((num_rel - num_rel_ret) as f64) * results_dcg / ideal_dcg;
                }

                let score = if num_rel > 0 { sum / (num_rel as f64) } else { 0.0 };
                vec![MetricValue::Float(score)]
    }
}


impl Default for NdcgRelMeasure {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_rel_standard() {
        let measure = NdcgRelMeasure::new("");
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
    }
}
