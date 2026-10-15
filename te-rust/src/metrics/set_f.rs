use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct SetFMeasure {
    beta: f64,
    sub_metrics: Vec<String>,
}

impl SetFMeasure {
    pub fn new(beta: f64, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "set_F".to_string()
        } else {
            format!("set_F_{}", params_str)
        };
        Self { beta, sub_metrics: vec![name] }
    }
}

impl Measure for SetFMeasure {
    fn name(&self) -> &'static str {
        "set_F"
    }

    fn short_description(&self) -> &'static str {
        "Set F measure"
    }

    fn explanation(&self) -> &'static str {
        "Set F measure: weighted harmonic mean of recall and precision\n\
        set_Fx = (x+1) * P * R / (R + x*P)\n\
        where x is the relative importance of R to P (default 1.0)."
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
        if let Some(q_state) = state.get_standard() {
            let score = if q_state.num_rel_ret > 0 && q_state.num_ret > 0 && q_state.num_rel > 0 {
                let p = (q_state.num_rel_ret as f64) / (q_state.num_ret as f64);
                let r = (q_state.num_rel_ret as f64) / (q_state.num_rel as f64);
                let denominator = self.beta * p + r;
                if denominator > 0.0 {
                    (self.beta + 1.0) * p * r / denominator
                } else {
                    0.0
                }
            } else {
                0.0
            };
            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for SetFMeasure {
    fn default() -> Self {
        Self::new(1.0, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_set_f_standard() {
        let measure = SetFMeasure::new(1.0, "");
        let config = EvalConfig::default();
        // retrieved: 3, relevant: 2, relevant retrieved: 2.
        // P = 2 / 3, R = 2 / 2 = 1.0.
        // score: 2.0 * (2/3) * 1.0 / (1.0 * 2/3 + 1.0) = (4/3) / (5/3) = 4 / 5 = 0.8.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.8)]);
    }

    #[test]
    fn test_set_f_custom_beta() {
        let measure = SetFMeasure::new(0.5, "0.5");
        let config = EvalConfig::default();
        // P = 2 / 3, R = 1.0.
        // beta = 0.5.
        // score: 1.5 * (2/3) * 1.0 / (0.5 * 2/3 + 1.0) = 1.0 / (4/3) = 3 / 4 = 0.75.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.75)]);
    }
}
