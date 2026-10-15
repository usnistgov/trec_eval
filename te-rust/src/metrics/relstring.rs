use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RelstringMeasure {
    len: usize,
    sub_metrics: Vec<String>,
}

impl RelstringMeasure {
    pub fn new(len: usize, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "relstring".to_string()
        } else {
            format!("relstring_{}", params_str)
        };
        Self { len, sub_metrics: vec![name] }
    }
}

impl Measure for RelstringMeasure {
    fn name(&self) -> &'static str {
        "relstring"
    }

    fn short_description(&self) -> &'static str {
        "Relevance character visualizer string"
    }

    fn explanation(&self) -> &'static str {
        "Relevance string for the first N retrieved docs (default 10). Displays '0'-'9' for relevance grades, '>' for grades > 9, '-' for non-pool unjudged, '.' for pool unjudged, and '<' otherwise. Only reported for individual queries."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Str
    }

    fn is_summary_enabled(&self) -> bool {
        false
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::NONE
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Str(String::new())]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let limit = std::cmp::min(self.len, q_state.results_rel_list.len());
        let mut s = String::with_capacity(self.len);

        for i in 0..limit {
            let rel = q_state.results_rel_list[i];
            let c = if rel > 9 {
                '>'
            } else if rel >= 0 {
                std::char::from_digit(rel as u32, 10).unwrap_or('<')
            } else if rel == crate::eval::alignment::RELVALUE_NONPOOL {
                '-'
            } else if rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                '.'
            } else {
                '<'
            };
            s.push(c);
        }

        let quoted = format!("'{}'", s);
        vec![MetricValue::Str(quoted)]
    }
}

impl Default for RelstringMeasure {
    fn default() -> Self {
        Self::new(30, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_relstring_standard() {
        let measure = RelstringMeasure::default();
        let config = EvalConfig::default();
        // retrieved: [1, 0, -1] (rel, nonrel, unjudged), len = 30
        let state = make_mock_state(vec![1, 0, -1], 1);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Str("'10-'".to_string())]);
    }

    #[test]
    fn test_relstring_empty_ranking() {
        let measure = RelstringMeasure::default();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Str("''".to_string())]);
    }
}
