use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use std::cmp::min;

pub struct RelativePMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl RelativePMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("relative_P_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for RelativePMeasure {
    fn name(&self) -> &'static str {
        "relative_P"
    }

    fn short_description(&self) -> &'static str {
        "Relative Precision at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Relative Precision at cutoff ranks. Calculates precision at cutoff relative to the maximum possible precision at that cutoff. This is equivalent to standard precision up until the total number of relevant documents (R), and then recall after R."
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

        for i in 0..q_state.num_ret {
            if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                let cutoff = self.cutoffs[cutoff_index];
                let max_possible = min(cutoff, q_state.num_rel);
                results[cutoff_index] = if max_possible > 0 {
                    (rel_so_far as f64) / (max_possible as f64)
                } else {
                    0.0
                };
                cutoff_index += 1;
                while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                    let cutoff = self.cutoffs[cutoff_index];
                    let max_possible = min(cutoff, q_state.num_rel);
                    results[cutoff_index] = if max_possible > 0 {
                        (rel_so_far as f64) / (max_possible as f64)
                    } else {
                        0.0
                    };
                    cutoff_index += 1;
                }
                if cutoff_index >= num_params {
                    break;
                }
            }

            if q_state.results_rel_list[i] >= config.relevance_level {
                rel_so_far += 1;
            }
        }

        while cutoff_index < num_params {
            let cutoff = self.cutoffs[cutoff_index];
            let max_possible = min(cutoff, q_state.num_rel);
            results[cutoff_index] = if max_possible > 0 {
                (rel_so_far as f64) / (max_possible as f64)
            } else {
                0.0
            };
            cutoff_index += 1;
        }

        results.into_iter().map(MetricValue::Float).collect()
    }
}

impl Default for RelativePMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_relative_p_standard() {
        let measure = RelativePMeasure::new(vec![1, 2, 3]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2
        // Rank 1: rel=1 -> rel_so_far=1. At i=1, relative_P@1 = rel_so_far / min(1, 2) = 1 / 1 = 1.0.
        // Rank 2: rel=0. At i=2, relative_P@2 = rel_so_far / min(2, 2) = 1 / 2 = 0.5.
        // Rank 3: rel=1 -> rel_so_far=2. At end, relative_P@3 = rel_so_far / min(3, 2) = 2 / 2 = 1.0.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0], MetricValue::Float(1.0));
        assert_eq!(actual[1], MetricValue::Float(0.5));
        assert_eq!(actual[2], MetricValue::Float(1.0));
    }
}
