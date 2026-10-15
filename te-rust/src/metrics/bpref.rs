use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct BprefMeasure;

impl BprefMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for BprefMeasure {
    fn name(&self) -> &'static str {
        "bpref"
    }

    fn short_description(&self) -> &'static str {
        "Binary Preference"
    }

    fn explanation(&self) -> &'static str {
        "Binary Preference. This measure computes the preference of relevant documents over non-relevant documents. It is based on the relative ranks of judged documents, calculating how often a relevant document is retrieved before a non-relevant document."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["bpref".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let mut num_nonrel = 0;
        for j in 0..config.relevance_level as usize {
            if j < q_state.rel_levels.len() {
                num_nonrel += q_state.rel_levels[j];
            }
        }

        let mut nonrel_so_far = 0;
        let mut bpref = 0.0;

        for &rel in &q_state.results_rel_list {
            if rel == crate::eval::alignment::RELVALUE_NONPOOL {
                continue;
            }
            if rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                continue;
            }

            if rel >= 0 && rel < config.relevance_level {
                nonrel_so_far += 1;
            } else {
                // Judged relevant document
                if nonrel_so_far > 0 {
                    let min_nonrel_num_rel = std::cmp::min(nonrel_so_far, q_state.num_rel as i64) as f64;
                    let min_total_nonrel_num_rel = std::cmp::min(num_nonrel, q_state.num_rel) as f64;
                    bpref += 1.0 - (min_nonrel_num_rel / min_total_nonrel_num_rel);
                } else {
                    bpref += 1.0;
                }
            }
        }

        if q_state.num_rel > 0 {
            bpref /= q_state.num_rel as f64;
        } else {
            bpref = 0.0;
        }

        vec![MetricValue::Float(bpref)]
    }
}

impl Default for BprefMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_bpref_standard() {
        let measure = BprefMeasure::new();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2. Non-relevant so far for rank 1: 0. bpref += 1.0.
        // Non-relevant so far for rank 3: 1 (at rank 2). min(1, 2)/min(100, 2) = 1/2 = 0.5. bpref += 1.0 - 0.5 = 0.5.
        // total bpref sum = 1.5. final score = 1.5 / 2 = 0.75.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.75)]);
    }
}
