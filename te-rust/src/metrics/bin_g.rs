use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct BinGMeasure;

impl BinGMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for BinGMeasure {
    fn name(&self) -> &'static str {
        "binG"
    }

    fn short_description(&self) -> &'static str {
        "Binary G"
    }

    fn explanation(&self) -> &'static str {
        "Binary G. Experimental measure. G is a gain related measure that combines qualities of MAP and NDCG. binG restricts the gain to either 0 or 1 (nonrel or rel), and thus is the average over all rel docs of (1 / log2 (2+num_nonrel before doc))."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["binG".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let mut sum = 0.0;
            let mut rel_so_far = 0;

            for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                if rel >= config.relevance_level {
                    rel_so_far += 1;
                    // Matches C: log2(3 + i - rel_so_far), i.e. 2 + (num nonrel retrieved
                    // before this doc). Compute in signed arithmetic to avoid underflow when
                    // the first document is relevant (i=0, rel_so_far=1 => arg = 2).
                    let arg = 3 + i as i64 - rel_so_far as i64;
                    sum += 1.0 / (arg as f64).log2();
                }
            }

            let score = if q_state.num_rel > 0 && rel_so_far > 0 {
                sum / (q_state.num_rel as f64)
            } else {
                0.0
            };

            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for BinGMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_bing_standard() {
        let measure = BinGMeasure::new();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2. C formula: 1 / log2(3 + i - rel_so_far).
        // rank 1 (i=0): rel=1 >= 1. rel_so_far = 1. num_nonrel before = 0. term = 1 / log2(2) = 1.0
        // rank 2 (i=1): rel=0.
        // rank 3 (i=2): rel=1 >= 1. rel_so_far = 2. num_nonrel before = 1. term = 1 / log2(3) = 0.63092975
        // sum = 1.0 + 0.63092975 = 1.63092975
        // score = sum / 2 = 0.815464875
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.815464875).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
