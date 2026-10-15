use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RunIdMeasure;

impl RunIdMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for RunIdMeasure {
    fn name(&self) -> &'static str {
        "runid"
    }

    fn short_description(&self) -> &'static str {
        "Run identifier"
    }

    fn explanation(&self) -> &'static str {
        "Run identifier. This represents the unique string identifier or tag assigned to the retrieval system run under evaluation."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Str
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["runid".to_string()]
    }

    fn is_query_enabled(&self) -> bool {
        false
    }

    fn is_summary_enabled(&self) -> bool {
        true
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::NONE
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Str("".to_string())]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            vec![MetricValue::Str(q_state.run_id.clone())]
        } else {
            self.initial_values()
        }
    }

    fn accumulate(&self, q_scores: &[MetricValue], running_totals: &mut [MetricValue]) {
        if let (MetricValue::Str(ref s), MetricValue::Str(ref mut t)) = (&q_scores[0], &mut running_totals[0]) {
            if t.is_empty() {
                *t = s.clone();
            }
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for runid
    }
}
