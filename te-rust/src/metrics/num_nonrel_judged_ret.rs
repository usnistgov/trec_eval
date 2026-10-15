use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NumNonrelJudgedRetMeasure;

impl NumNonrelJudgedRetMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumNonrelJudgedRetMeasure {
    fn name(&self) -> &'static str {
        "num_nonrel_judged_ret"
    }

    fn short_description(&self) -> &'static str {
        "Number of non-relevant judged retrieved"
    }

    fn explanation(&self) -> &'static str {
        "Number of non-relevant judged documents retrieved for topic. Not an evaluation number per se, but gives details of retrieval results. Summary figure is sum of individual topics, not average."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_nonrel_judged_ret".to_string()]
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::COUNTS
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let mut count = 0;
            for &rel in &q_state.results_rel_list {
                if rel >= 0 && rel < config.relevance_level {
                    count += 1;
                }
            }
            vec![MetricValue::Integer(count)]
        } else {
            self.initial_values()
        }
    }


    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for counts (reports overall sum across topics)
    }
}

impl Default for NumNonrelJudgedRetMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_num_nonrel_judged_ret_standard() {
        let measure = NumNonrelJudgedRetMeasure::new();
        let config = EvalConfig::default();
        // retrieved: [1, 0, -1, -2, 0]. relevance_level = 1.
        // Judged non-relevant are: 0 (index 1), 0 (index 4).
        // Total should be 2.
        let state = make_mock_state(vec![1, 0, -1, -2, 0], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Integer(2)]);
    }
}
