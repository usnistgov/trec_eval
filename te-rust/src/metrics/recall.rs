use crate::metrics::common::{validate_rank_cutoffs, MeasureParseError};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RecallCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl RecallCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Result<Self, MeasureParseError> {
        let cutoffs = validate_rank_cutoffs(cutoffs)?;
        let sub_metrics = cutoffs.iter().map(|c| format!("recall_{}", c)).collect();
        Ok(Self { cutoffs, sub_metrics })
    }
}

impl Measure for RecallCutMeasure {
    fn name(&self) -> &'static str {
        "recall"
    }

    fn short_description(&self) -> &'static str {
        "Recall at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Recall at cutoff ranks. Calculates the proportion of known relevant documents that have been retrieved by the system up to the specified cutoff ranks (e.g. top 5, 10, 15...)."
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
        vec![MetricValue::Float(0.0); self.cutoffs.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let mut results = Vec::with_capacity(self.cutoffs.len());
        let num_rel = q_state.num_rel;

        for &c in &self.cutoffs {
            let rel_ret = super::common::count_relevant_retrieved_up_to(&q_state.results_rel_list, c, config);
            let recall = if num_rel > 0 {
                (rel_ret as f64) / (num_rel as f64)
            } else {
                0.0
            };
            results.push(MetricValue::Float(recall));
        }

        results
    }
}

impl Default for RecallCutMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])
            .expect("default recall cutoffs are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_recall_standard() {
        let measure = RecallCutMeasure::new(vec![1, 3]).unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 4.
        // Recall@1 = 1 / 4 = 0.25. Recall@3 = 2 / 4 = 0.5.
        let state = make_mock_state(vec![1, 0, 1], 4);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.25), MetricValue::Float(0.5)]);
    }

    #[test]
    fn test_recall_unsorted_and_invalid_cutoffs() {
        let measure = RecallCutMeasure::new(vec![5, 2]).unwrap();
        assert_eq!(measure.sub_metrics(), vec!["recall_2", "recall_5"]);
        let config = EvalConfig::default();
        let state = EvalState::Standard(make_mock_state(vec![1, 1, 1, 1, 1, 1], 6));
        let actual = measure.calc(&config, &state);
        assert_eq!(actual, vec![MetricValue::Float(2.0 / 6.0), MetricValue::Float(5.0 / 6.0)]);
        assert!(RecallCutMeasure::new(vec![]).is_err());
        assert!(RecallCutMeasure::new(vec![0]).is_err());
        assert!(RecallCutMeasure::new(vec![5, 5]).is_err());
        assert!(RecallCutMeasure::new(vec![i64::MAX as usize + 1]).is_err());
    }
}
