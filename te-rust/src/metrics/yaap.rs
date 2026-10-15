use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct YaapMeasure;

impl YaapMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for YaapMeasure {
    fn name(&self) -> &'static str {
        "yaap"
    }

    fn short_description(&self) -> &'static str {
        "Yet Another Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Yet Another Average Precision. Robertson's smoothed adaptation of MAP to produce values that are more globally averageable than MAP."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["yaap".to_string()]
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::FINITE_ONLY
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
                    sum += (rel_so_far as f64) / (i + 1) as f64;
                }
            }

            let num_rel = q_state.num_rel as f64;
            let val = ((1.0 + sum) / (1.0 + num_rel - sum)).ln();

            vec![MetricValue::Float(val)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for YaapMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_yaap_standard() {
        let measure = YaapMeasure::new();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2.
        // sum = 1.0/1 + 2.0/3 = 1.6666667.
        // val = ln((1.0 + 1.6666667) / (1.0 + 2.0 - 1.6666667)) = ln(2.6666667 / 1.3333333) = ln(2.0) = 0.69314718
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 2.0_f64.ln()).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_yaap_empty_ranking() {
        let measure = YaapMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        // sum = 0.0. R = 5. val = ln(1.0 / 6.0) = -1.791759
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - (1.0_f64 / 6.0).ln()).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }
}
