use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct GMBprefMeasure;

impl GMBprefMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for GMBprefMeasure {
    fn name(&self) -> &'static str {
        "gm_bpref"
    }

    fn short_description(&self) -> &'static str {
        "Geometric Mean bpref"
    }

    fn explanation(&self) -> &'static str {
        "Binary preference (bpref), but using geometric mean over topics. gm_bpref is printed only as a summary measure across topics, not for the individual topics."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["gm_bpref".to_string()]
    }

    fn is_query_enabled(&self) -> bool {
        false
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::FINITE_ONLY
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

        let val = bpref.max(0.00001).ln();
        vec![MetricValue::Float(val)]
    }

    fn average(&self, config: &EvalConfig, running_totals: &mut [MetricValue], num_queries_evaluated: usize, total_qrels_queries: usize) {
        let denominator = if config.average_complete_flag {
            total_qrels_queries
        } else {
            num_queries_evaluated
        };
        if denominator > 0 {
            for total in running_totals.iter_mut() {
                if let MetricValue::Float(t) = total {
                    if config.average_complete_flag {
                        let missing = total_qrels_queries as f64 - num_queries_evaluated as f64;
                        if missing > 0.0 {
                            *t += missing * (0.00001_f64).ln();
                        }
                    }
                    *t = (*t / denominator as f64).exp();
                }
            }
        }
    }
}

impl Default for GMBprefMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_gm_bpref_standard() {
        let measure = GMBprefMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - (0.75_f64).ln()).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_gm_bpref_empty_ranking() {
        let measure = GMBprefMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float((0.00001_f64).ln())]);
    }
}
