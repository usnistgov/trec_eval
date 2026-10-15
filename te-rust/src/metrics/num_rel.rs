use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NumRelMeasure;

impl NumRelMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumRelMeasure {
    fn name(&self) -> &'static str {
        "num_rel"
    }

    fn short_description(&self) -> &'static str {
        "Total relevant"
    }

    fn explanation(&self) -> &'static str {
        "Total number of relevant documents. This is the count of documents that are judged relevant in the ground-truth relevance judgments (qrels) for the given topic."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_rel".to_string()]
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::COUNTS
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            vec![MetricValue::Integer(q_state.num_rel as i64)]
        } else {
            self.initial_values()
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for counts
    }
}

impl Default for NumRelMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_num_rel_standard() {
        let measure = NumRelMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Integer(5)]);
    }

    #[test]
    fn test_num_rel_zero() {
        let measure = NumRelMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 1], 0);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Integer(0)]);
    }
}
