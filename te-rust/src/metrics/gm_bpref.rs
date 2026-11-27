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

        let num_nonrel: usize = q_state
            .rel_levels
            .iter()
            .take_while(|(&level, _)| level < config.relevance_level)
            .filter(|(&level, _)| level >= 0)
            .map(|(_, &count)| count)
            .sum();

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

    fn average(&self, running_totals: &mut [MetricValue], num_queries: usize) {
        if num_queries > 0 {
            for total in running_totals.iter_mut() {
                if let MetricValue::Float(t) = total {
                    *t = (*t / num_queries as f64).exp();
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

    #[test]
    fn test_gm_bpref_average_with_empty_topic_and_zero_queries() {
        let measure = GMBprefMeasure::new();
        let config = EvalConfig::default();
        let mut totals = measure.initial_values();
        for state in [
            make_mock_state(vec![1], 1),    // bpref = 1.0
            make_mock_state(vec![1, 0], 2), // bpref = 0.5
            make_mock_state(vec![], 1),     // missing/empty -> floor 1e-5
        ] {
            let scores = measure.calc(&config, &EvalState::Standard(state));
            measure.accumulate(&scores, &mut totals);
        }
        measure.average(&mut totals, 3);
        let expected = ((1.0_f64.ln() + 0.5_f64.ln() + 1e-5_f64.ln()) / 3.0).exp();
        if let MetricValue::Float(v) = totals[0] {
            assert!((v - expected).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }

        let mut empty_totals = measure.initial_values();
        measure.average(&mut empty_totals, 0);
        assert_eq!(empty_totals, vec![MetricValue::Float(0.0)]);
    }

    #[test]
    fn test_gm_bpref_extreme_relevance_levels() {
        use crate::eval::alignment::align_query;
        use crate::io::{QrelsQuery, QrelsRecord, RunQuery, RunRecord};

        let measure = GMBprefMeasure::new();
        let qrels = QrelsQuery {
            qid: "1".to_string(),
            records: vec![
                QrelsRecord { docno: "a".to_string(), rel: 2 },
                QrelsRecord { docno: "b".to_string(), rel: 0 },
            ],
        };
        let run = RunQuery {
            qid: "1".to_string(),
            records: vec![
                RunRecord { docno: "a".to_string(), sim: 2.0 },
                RunRecord { docno: "b".to_string(), sim: 1.0 },
            ],
        };

        for lvl in [4, 1_000_000_000_000, i64::MAX] {
            let mut config = EvalConfig::default();
            config.relevance_level = lvl;
            let state = align_query(Some(&run), &qrels, "r", lvl, usize::MAX, false);
            assert_eq!(
                measure.calc(&config, &EvalState::Standard(state)),
                vec![MetricValue::Float((0.00001_f64).ln())]
            );
        }

        for lvl in [0, -1] {
            let mut config = EvalConfig::default();
            config.relevance_level = lvl;
            let state = align_query(Some(&run), &qrels, "r", lvl, usize::MAX, false);
            assert_eq!(
                measure.calc(&config, &EvalState::Standard(state)),
                vec![MetricValue::Float((1.0_f64).ln())]
            );
        }
    }
}
