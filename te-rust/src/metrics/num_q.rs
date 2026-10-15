use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NumQMeasure;

impl NumQMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumQMeasure {
    fn name(&self) -> &'static str {
        "num_q"
    }

    fn short_description(&self) -> &'static str {
        "Total queries"
    }

    fn explanation(&self) -> &'static str {
        "Number of topics results averaged over. May be different from number of topics in the results file if -c was used on the command line in which case number of topics in the rel_info file is used."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_q".to_string()]
    }

    fn is_query_enabled(&self) -> bool {
        false
    }

    fn is_summary_enabled(&self) -> bool {
        true
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::COUNTS
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, _config: &EvalConfig, _state: &EvalState) -> Vec<MetricValue> {
        vec![MetricValue::Integer(1)]
    }

    fn average(
        &self,
        config: &EvalConfig,
        running_totals: &mut [MetricValue],
        num_queries_evaluated: usize,
        total_qrels_queries: usize,
    ) {
        let count = if config.average_complete_flag {
            total_qrels_queries
        } else {
            num_queries_evaluated
        };
        running_totals[0] = MetricValue::Integer(count as i64);
    }
}

impl Default for NumQMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_num_q_standard() {
        let measure = NumQMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Integer(1)]);
        assert!(!measure.is_query_enabled());
        assert!(measure.is_summary_enabled());

        let mut totals = measure.initial_values();
        measure.average(&config, &mut totals, 42, 50);
        assert_eq!(totals, vec![MetricValue::Integer(42)]);

        let mut complete_config = EvalConfig::default();
        complete_config.average_complete_flag = true;
        let mut totals_c = measure.initial_values();
        measure.average(&complete_config, &mut totals_c, 42, 50);
        assert_eq!(totals_c, vec![MetricValue::Integer(50)]);
    }
}
