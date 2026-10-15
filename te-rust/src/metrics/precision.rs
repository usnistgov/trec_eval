use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct PrecisionCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl PrecisionCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("P_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for PrecisionCutMeasure {
    fn name(&self) -> &'static str {
        "P"
    }

    fn short_description(&self) -> &'static str {
        "Precision at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Precision at cutoff ranks. Calculates the proportion of retrieved documents that are relevant, evaluated at specific rank thresholds (e.g. top 5, 10, 15... retrieved documents). It measures retrieval accuracy at specified system output depths."
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
            let precision = (rel_ret as f64) / (c as f64);
            results.push(MetricValue::Float(precision));
        }

        results
    }
}

impl Default for PrecisionCutMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_precision_standard() {
        let measure = PrecisionCutMeasure::new(vec![1, 3]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // P@1 = 1 / 1 = 1.0. P@3 = 2 / 3 = 0.6667.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0], MetricValue::Float(1.0));
        if let MetricValue::Float(v) = actual[1] {
            assert!((v - 0.6666666666666666).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }
}
