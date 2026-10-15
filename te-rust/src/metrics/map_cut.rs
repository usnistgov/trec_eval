use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct MapCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl MapCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("map_cut_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for MapCutMeasure {
    fn name(&self) -> &'static str {
        "map_cut"
    }

    fn short_description(&self) -> &'static str {
        "Mean Average Precision at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Mean Average Precision at cutoff ranks. Average precision measured at specified document cutoff thresholds in the ranking. It represents the sum of precisions after each relevant document retrieved up to the cutoff rank, divided by the total number of relevant documents for that topic. If the cutoff is larger than the retrieved set size, unretrieved documents are assumed non-relevant."
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

        let num_params = self.cutoffs.len();
        let mut results = vec![0.0; num_params];

        if q_state.num_rel == 0 {
            return results.into_iter().map(MetricValue::Float).collect();
        }

        let mut cutoff_index = 0;
        let mut rel_so_far = 0;
        let mut sum_prec = 0.0;

        for i in 0..q_state.num_ret {
            if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                results[cutoff_index] = sum_prec / (q_state.num_rel as f64);
                cutoff_index += 1;
                while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                    results[cutoff_index] = sum_prec / (q_state.num_rel as f64);
                    cutoff_index += 1;
                }
                if cutoff_index >= num_params {
                    break;
                }
            }

            if q_state.results_rel_list[i] >= config.relevance_level {
                rel_so_far += 1;
                sum_prec += (rel_so_far as f64) / ((i + 1) as f64);
            }
        }

        while cutoff_index < num_params {
            results[cutoff_index] = sum_prec / (q_state.num_rel as f64);
            cutoff_index += 1;
        }

        results.into_iter().map(MetricValue::Float).collect()
    }

}

impl Default for MapCutMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_map_cut_standard() {
        let measure = MapCutMeasure::new(vec![1, 2, 3]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2
        // Rank 1: rel=1 -> rel_so_far=1, prec=1/1=1.0, sum_prec=1.0. At i=1 (cutoff 1 achieved at i=1?),
        // Wait, cutoffs are 1, 2, 3.
        // i=0: doc is relevant. rel_so_far=1, sum_prec = 1.0.
        // i=1 (cutoff_index=0 achieved): map_cut_1 = sum_prec / 2 = 1.0 / 2 = 0.5.
        //   doc is nonrelevant. sum_prec = 1.0.
        // i=2 (cutoff_index=1 achieved): map_cut_2 = sum_prec / 2 = 1.0 / 2 = 0.5.
        //   doc is relevant. rel_so_far=2, prec=2/3=0.666667, sum_prec = 1.0 + 2/3 = 1.666667.
        // loop finishes:
        // map_cut_3 = sum_prec / 2 = 1.666667 / 2 = 0.833333.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0], MetricValue::Float(0.5));
        assert_eq!(actual[1], MetricValue::Float(0.5));
        if let MetricValue::Float(v) = actual[2] {
            assert!((v - 0.83333333).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
