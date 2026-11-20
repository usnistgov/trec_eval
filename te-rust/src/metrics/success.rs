use crate::metrics::common::{validate_rank_cutoffs, MeasureParseError};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct SuccessCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl SuccessCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Result<Self, MeasureParseError> {
        let cutoffs = validate_rank_cutoffs(cutoffs)?;
        let sub_metrics = cutoffs.iter().map(|c| format!("success_{}", c)).collect();
        Ok(Self { cutoffs, sub_metrics })
    }
}

impl Measure for SuccessCutMeasure {
    fn name(&self) -> &'static str {
        "success"
    }

    fn short_description(&self) -> &'static str {
        "Success at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Success at cutoff ranks. Calculates whether at least one relevant document was retrieved by the system up to the specified rank thresholds. It is represented as 1.0 (success) or 0.0 (failure)."
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

        for &c in &self.cutoffs {
            let rel_ret = super::common::count_relevant_retrieved_up_to(&q_state.results_rel_list, c, config);
            let success = if rel_ret > 0 { 1.0 } else { 0.0 };
            results.push(MetricValue::Float(success));
        }

        results
    }
}

impl Default for SuccessCutMeasure {
    fn default() -> Self {
        Self::new(vec![1, 5, 10]).expect("default success cutoffs are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_success_standard() {
        let measure = SuccessCutMeasure::new(vec![1, 3]).unwrap();
        let config = EvalConfig::default();
        // retrieved: [0, 0, 1] (relevant at rank 3). num_rel = 1.
        // Success@1 = 0.0. Success@3 = 1.0.
        let state = make_mock_state(vec![0, 0, 1], 1);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0), MetricValue::Float(1.0)]);
    }

    #[test]
    fn test_success_unsorted_and_invalid_cutoffs() {
        let measure = SuccessCutMeasure::new(vec![3, 1]).unwrap();
        assert_eq!(measure.sub_metrics(), vec!["success_1", "success_3"]);
        assert!(SuccessCutMeasure::new(vec![]).is_err());
        assert!(SuccessCutMeasure::new(vec![0]).is_err());
        assert!(SuccessCutMeasure::new(vec![5, 5]).is_err());
        assert!(SuccessCutMeasure::new(vec![i64::MAX as usize + 1]).is_err());
    }
}
