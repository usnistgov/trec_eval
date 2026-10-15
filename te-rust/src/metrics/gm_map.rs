use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct GMMapMeasure;

impl GMMapMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for GMMapMeasure {
    fn name(&self) -> &'static str {
        "gm_map"
    }

    fn short_description(&self) -> &'static str {
        "Geometric Mean Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Geometric Mean Average Precision. This is the same measure as 'map' on an individual topic, but the geometric mean is calculated when averaging over topics. This rewards methods that are more consistent over topics as opposed to methods which do very well for some topics but very poorly for others. gm_map is reported only in the summary over all topics."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["gm_map".to_string()]
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
        if let Some(q_state) = state.get_standard() {
            let mut rel_so_far = 0;
            let mut sum = 0.0;

            for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                if rel >= config.relevance_level {
                    rel_so_far += 1;
                    sum += (rel_so_far as f64) / (i + 1) as f64;
                }
            }

            let ap = if q_state.num_rel > 0 && rel_so_far > 0 {
                sum / (q_state.num_rel as f64)
            } else {
                0.0
            };

            let val = ap.max(0.00001).ln();
            vec![MetricValue::Float(val)]
        } else {
            self.initial_values()
        }
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

impl Default for GMMapMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_gm_map_standard() {
        let measure = GMMapMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            let expected_ap: f64 = 0.8333333333333333;
            assert!((v - expected_ap.ln()).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_gm_map_empty_ranking() {
        let measure = GMMapMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float((0.00001_f64).ln())]);
    }
}
