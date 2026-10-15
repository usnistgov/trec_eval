use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct UtilityMeasure {
    params: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl UtilityMeasure {
    pub fn new(params: Vec<f64>, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "utility".to_string()
        } else {
            format!("utility_{}", params_str)
        };
        Self { params, sub_metrics: vec![name] }
    }
}

impl Measure for UtilityMeasure {
    fn name(&self) -> &'static str {
        "utility"
    }

    fn short_description(&self) -> &'static str {
        "Set utility measure"
    }

    fn explanation(&self) -> &'static str {
        "Utility based on a contingency table of retrieved vs relevant docs. Computed as p1*a + p2*b + p3*c + p4*d, where a is retrieved-relevant, b is retrieved-nonrelevant, c is nonretrieved-relevant, d is nonretrieved-nonrelevant, and p1-p4 are weights (default: 1.0, -1.0, 0.0, 0.0)."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::SIGNED
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let num_ret = q_state.results_rel_list.len();

        // Count total relevant retrieved
        let mut num_rel_ret = 0;
        for &rel in &q_state.results_rel_list {
            if rel >= config.relevance_level {
                num_rel_ret += 1;
            }
        }

        let a = num_rel_ret as f64;
        let b = (num_ret - num_rel_ret) as f64;
        let c = (q_state.num_rel - num_rel_ret) as f64;
        
        // d = num_docs_in_coll + num_rel_ret - num_ret - num_rel
        let d = (config.num_docs_in_coll as i64 + num_rel_ret as i64 - num_ret as i64 - q_state.num_rel as i64) as f64;

        let score = self.params[0] * a +
                    self.params[1] * b +
                    self.params[2] * c +
                    self.params[3] * d;

        vec![MetricValue::Float(score)]
    }
}

impl Default for UtilityMeasure {
    fn default() -> Self {
        Self::new(vec![1.0, -1.0, 0.0, 0.0], "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_utility_standard() {
        let measure = UtilityMeasure::default();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1] (2 relevant, 1 non-relevant), num_rel = 2.
        // default params: [1.0, -1.0, 0.0, 0.0]
        // score = 1.0 * 2 + (-1.0) * 1 = 1.0
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(1.0)]);
    }

    #[test]
    fn test_utility_zero_relevance() {
        let measure = UtilityMeasure::default();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![0, 0], 0);
        // a = 0, b = 2, c = 0, d = num_docs_in_coll + 0 - 2 - 0 = num_docs_in_coll - 2.
        // score = 1.0 * 0 + (-1.0) * 2 = -2.0
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(-2.0)]);
    }
}
