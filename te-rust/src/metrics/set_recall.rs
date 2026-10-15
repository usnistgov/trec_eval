use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct SetRecallMeasure;

impl SetRecallMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for SetRecallMeasure {
    fn name(&self) -> &'static str {
        "set_recall"
    }

    fn short_description(&self) -> &'static str {
        "Set Recall"
    }

    fn explanation(&self) -> &'static str {
        "Set Recall: num_relevant_retrieved / num_relevant\n\
        Recall over all docs retrieved for a topic."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["set_recall".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let score = if q_state.num_rel > 0 {
                (q_state.num_rel_ret as f64) / (q_state.num_rel as f64)
            } else {
                0.0
            };
            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for SetRecallMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_set_recall_standard() {
        let measure = SetRecallMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 4);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(2.0 / 4.0)]);
    }

    #[test]
    fn test_set_recall_no_relevant() {
        let measure = SetRecallMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![0, 0], 0);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0)]);
    }
}
