use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RbpResidMeasure {
    p: f64,
    _params_str: String,
    sub_metrics: Vec<String>,
}

impl RbpResidMeasure {
    pub fn new(p: f64, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "rbp_resid".to_string()
        } else {
            format!("rbp_resid_{}", params_str)
        };
        Self {
            p,
            _params_str: params_str.to_string(),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for RbpResidMeasure {
    fn name(&self) -> &'static str {
        "rbp_resid"
    }

    fn short_description(&self) -> &'static str {
        "Rank-Biased Precision residual"
    }

    fn explanation(&self) -> &'static str {
        "Rank-Biased Precision residual. Computes the maximum possible upward shift in RBP score if all unjudged retrieved documents were actually relevant."
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
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let mut unj_so_far = 0;
        let mut sum = 0.0;
        let mut cur_p = 1.0;

        for &rel in &q_state.results_rel_list {
            if rel < 0 {
                unj_so_far += 1;
                sum += cur_p;
            }
            cur_p *= self.p;
        }

        let val = if unj_so_far > 0 {
            cur_p + (1.0 - self.p) * sum
        } else {
            0.0
        };

        vec![MetricValue::Float(val)]
    }
}

impl Default for RbpResidMeasure {
    fn default() -> Self {
        Self::new(0.9, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rbp_resid_standard() {
        let measure = RbpResidMeasure::new(0.9, "");
        let config = EvalConfig::default();
        // retrieved: [1, -1, 1] (index 1 is unjudged).
        // loop:
        // i=0: rel=1 >= 0. cur_p = 1.0 -> 0.9
        // i=1: rel=-1 < 0. unj_so_far = 1, sum += 0.9. cur_p = 0.9 -> 0.81
        // i=2: rel=1 >= 0. cur_p = 0.81 -> 0.729
        // total sum = 0.9. unj_so_far = 1.
        // val = cur_p + (1 - p) * sum = 0.729 + 0.1 * 0.9 = 0.729 + 0.09 = 0.819
        let state = make_mock_state(vec![1, -1, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.819).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_rbp_resid_zero_unjudged() {
        let measure = RbpResidMeasure::new(0.9, "");
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0)]);
    }
}
