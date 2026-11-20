use crate::metrics::common::{parse_key_values, MeasureParseError};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RbpResidMeasure {
    p: f64,
    sub_metrics: Vec<String>,
}

impl RbpResidMeasure {
    pub fn new(p: f64, params_str: &str) -> Result<Self, MeasureParseError> {
        if !p.is_finite() {
            return Err(MeasureParseError::new(format!(
                "non-finite rbp_resid persistence '{}'",
                p
            )));
        }
        if !(0.0..1.0).contains(&p) {
            return Err(MeasureParseError::new(format!(
                "rbp_resid persistence '{}' must be in [0.0, 1.0)",
                p
            )));
        }
        let name = if params_str.is_empty() {
            "rbp_resid".to_string()
        } else {
            format!("rbp_resid_{}", params_str)
        };
        Ok(Self {
            p,
            sub_metrics: vec![name],
        })
    }

    pub fn from_params(params: Option<&str>) -> Result<Self, MeasureParseError> {
        let Some(params_str) = params else {
            return Self::new(0.9, "");
        };
        let pairs = parse_key_values(params_str)?;
        if pairs.is_empty() {
            return Err(MeasureParseError::new("rbp_resid parameter string must not be empty"));
        }
        let mut p = 0.9;
        let mut seen_p = false;
        for (k, v) in pairs {
            if k == "p" {
                if seen_p {
                    return Err(MeasureParseError::new("duplicate rbp_resid parameter 'p'"));
                }
                seen_p = true;
                p = v.parse::<f64>().map_err(|_| {
                    MeasureParseError::new(format!("invalid float parameter '{}'", v))
                })?;
            } else {
                return Err(MeasureParseError::new(format!(
                    "unknown rbp_resid parameter '{}'",
                    k
                )));
            }
        }
        Self::new(p, params_str)
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
        Self::new(0.9, "").expect("default rbp_resid parameter is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rbp_resid_standard() {
        let measure = RbpResidMeasure::new(0.9, "").unwrap();
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
        let measure = RbpResidMeasure::new(0.9, "").unwrap();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0)]);
    }

    #[test]
    fn test_rbp_resid_params_validation() {
        let default_m = RbpResidMeasure::from_params(None).unwrap();
        assert_eq!(default_m.sub_metrics(), vec!["rbp_resid"]);

        let zero_p = RbpResidMeasure::from_params(Some("p=0.0")).unwrap();
        assert_eq!(zero_p.sub_metrics(), vec!["rbp_resid_p=0.0"]);

        // Rejects invalid p values
        assert!(RbpResidMeasure::new(f64::NAN, "p=nan").is_err());
        assert!(RbpResidMeasure::new(f64::INFINITY, "p=inf").is_err());
        assert!(RbpResidMeasure::new(f64::NEG_INFINITY, "p=-inf").is_err());
        assert!(RbpResidMeasure::new(-0.1, "p=-0.1").is_err());
        assert!(RbpResidMeasure::new(1.0, "p=1.0").is_err());
        assert!(RbpResidMeasure::new(1.5, "p=1.5").is_err());

        // Rejects malformed, unknown, gain-key, and duplicate params
        assert!(RbpResidMeasure::from_params(Some("")).is_err());
        assert!(RbpResidMeasure::from_params(Some("0.5")).is_err());
        assert!(RbpResidMeasure::from_params(Some("q=0.5")).is_err());
        assert!(RbpResidMeasure::from_params(Some("1=2")).is_err());
        assert!(RbpResidMeasure::from_params(Some("p=0.5,1=2")).is_err());
        assert!(RbpResidMeasure::from_params(Some("p=0.5,p=0.8")).is_err());
        assert!(RbpResidMeasure::from_params(Some("p=bad")).is_err());
        assert!(RbpResidMeasure::from_params(Some("p=1.0")).is_err());
    }
}
