use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct SetRelativePMeasure;

impl SetRelativePMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for SetRelativePMeasure {
    fn name(&self) -> &'static str {
        "set_relative_P"
    }

    fn short_description(&self) -> &'static str {
        "Relative Set Precision"
    }

    fn explanation(&self) -> &'static str {
        "Relative Set Precision: P / (Max possible P for this size set)\n\
        Relative precision over all docs retrieved for a topic."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["set_relative_P".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let score = if q_state.num_ret > 0 && q_state.num_rel > 0 {
                let max_possible = std::cmp::min(q_state.num_ret, q_state.num_rel);
                (q_state.num_rel_ret as f64) / (max_possible as f64)
            } else {
                0.0
            };
            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for SetRelativePMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_set_relative_p_standard() {
        let measure = SetRelativePMeasure::new();
        let config = EvalConfig::default();
        // retrieved: 3, relevant: 2, relevant retrieved: 1.
        // max possible: min(3, 2) = 2.
        // score: 1 / 2 = 0.5.
        let state = make_mock_state(vec![1, 0, 0], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.5)]);
    }
}
