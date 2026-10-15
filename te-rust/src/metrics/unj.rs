use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct UnjMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl UnjMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("unj_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for UnjMeasure {
    fn name(&self) -> &'static str {
        "unj"
    }

    fn short_description(&self) -> &'static str {
        "Unjudged at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Unjudged at cutoffs. The fraction of unjudged documents measured at various doc level cutoffs in the ranking. If the cutoff is larger than the number of docs retrieved, then it is assumed irrelevant (not unjudged) docs fill in the rest."
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

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let mut results = vec![0.0; self.cutoffs.len()];
        let mut cutoff_index = 0;
        let mut unj_so_far = 0;

        for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
            if cutoff_index < self.cutoffs.len() && i == self.cutoffs[cutoff_index] {
                results[cutoff_index] = (unj_so_far as f64) / (i as f64);
                cutoff_index += 1;
                while cutoff_index < self.cutoffs.len() && i == self.cutoffs[cutoff_index] {
                    results[cutoff_index] = (unj_so_far as f64) / (i as f64);
                    cutoff_index += 1;
                }
            }
            if rel == crate::eval::alignment::RELVALUE_NONPOOL || rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                unj_so_far += 1;
            }
        }

        while cutoff_index < self.cutoffs.len() {
            results[cutoff_index] = (unj_so_far as f64) / (self.cutoffs[cutoff_index] as f64);
            cutoff_index += 1;
        }

        results.into_iter().map(MetricValue::Float).collect()
    }
}

impl Default for UnjMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 20])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_unj_standard() {
        let measure = UnjMeasure::new(vec![2, 5]);
        let config = EvalConfig::default();
        // retrieved: [-1, 1, -2, 0, 0]. First is unjudged (NONPOOL), third is unjudged (UNJUDGED).
        // unj_2: unj_so_far at i=2 is 1 (NONPOOL at index 0). result = 1 / 2 = 0.5.
        // unj_5: unj_so_far at end is 2 (NONPOOL at 0, UNJUDGED at 2). result = 2 / 5 = 0.4.
        let state = make_mock_state(vec![-1, 1, -2, 0, 0], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.5), MetricValue::Float(0.4)]);
    }
}
