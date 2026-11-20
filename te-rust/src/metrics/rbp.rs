use crate::metrics::common::{parse_key_values, Gains, GainsConfig, MeasureParseError};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use std::collections::HashMap;

pub struct RbpMeasure {
    p: f64,
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl RbpMeasure {
    pub fn new(p: f64, gains_config: GainsConfig, params_str: &str) -> Result<Self, MeasureParseError> {
        if !p.is_finite() {
            return Err(MeasureParseError::new(format!(
                "non-finite rbp persistence '{}'",
                p
            )));
        }
        if !(0.0..1.0).contains(&p) {
            return Err(MeasureParseError::new(format!(
                "rbp persistence '{}' must be in [0.0, 1.0)",
                p
            )));
        }
        for (&rel, &gain) in &gains_config.custom_gains {
            if !gain.is_finite() {
                return Err(MeasureParseError::new(format!(
                    "non-finite rbp gain '{}' for relevance level '{}'",
                    gain, rel
                )));
            }
        }
        let name = if params_str.is_empty() {
            "rbp".to_string()
        } else {
            format!("rbp_{}", params_str)
        };
        Ok(Self {
            p,
            gains_config,
            sub_metrics: vec![name],
        })
    }

    pub fn from_params(params: Option<&str>) -> Result<Self, MeasureParseError> {
        let Some(params_str) = params else {
            return Self::new(0.9, GainsConfig::default(), "");
        };
        let pairs = parse_key_values(params_str)?;
        if pairs.is_empty() {
            return Err(MeasureParseError::new("rbp parameter string must not be empty"));
        }
        let mut p = 0.9;
        let mut seen_p = false;
        let mut custom_gains = HashMap::new();
        for (k, v) in pairs {
            if k == "p" {
                if seen_p {
                    return Err(MeasureParseError::new("duplicate rbp parameter 'p'"));
                }
                seen_p = true;
                p = v.parse::<f64>().map_err(|_| {
                    MeasureParseError::new(format!("invalid float parameter '{}'", v))
                })?;
            } else if let Ok(rel_level) = k.parse::<i64>() {
                if custom_gains.contains_key(&rel_level) {
                    return Err(MeasureParseError::new(format!(
                        "duplicate relevance level '{}'",
                        rel_level
                    )));
                }
                let gain = v.parse::<f64>().map_err(|_| {
                    MeasureParseError::new(format!(
                        "invalid gain value '{}' for relevance level '{}'",
                        v, k
                    ))
                })?;
                custom_gains.insert(rel_level, gain);
            } else {
                return Err(MeasureParseError::new(format!(
                    "unknown rbp parameter '{}'",
                    k
                )));
            }
        }
        Self::new(p, GainsConfig { custom_gains }, params_str)
    }
}

impl Measure for RbpMeasure {
    fn name(&self) -> &'static str {
        "rbp"
    }

    fn short_description(&self) -> &'static str {
        "Rank-Biased Precision"
    }

    fn explanation(&self) -> &'static str {
        "Rank-Biased Precision. RBP measures the rate at which utility is gained by a user working at a given degree of persistence p. By default, p = 0.9. Higher values of p simulate a more persistent user who reads deeper into the ranking list."
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

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let gains_config = if !self.gains_config.custom_gains.is_empty() {
            &self.gains_config
        } else if let Some(ref gg) = config.global_gains {
            gg
        } else {
            &self.gains_config
        };

        let gains = Gains::setup(gains_config, &q_state.rel_levels);

        // Find min and max gains to normalize if needed
        let mut min_gain = f64::INFINITY;
        let mut max_gain = f64::NEG_INFINITY;
        for g in &gains.rel_gains {
            if g.gain < min_gain {
                min_gain = g.gain;
            }
            if g.gain > max_gain {
                max_gain = g.gain;
            }
        }

        let get_normalized_gain = |rel_level: i64| -> f64 {
            let raw_gain = gains.get_gain(rel_level);
            if min_gain < 0.0 || max_gain > 1.0 {
                let range = max_gain - min_gain;
                if range > 0.0 {
                    (raw_gain - min_gain) / range
                } else {
                    0.0
                }
            } else {
                raw_gain
            }
        };

        let mut sum = 0.0;
        let mut cur_p = 1.0;

        for &rel in &q_state.results_rel_list {
            let gain = get_normalized_gain(rel);
            if gain != 0.0 {
                sum += gain * cur_p;
            }
            cur_p *= self.p;
        }

        let val = (1.0 - self.p) * sum;
        vec![MetricValue::Float(val)]
    }
}

impl Default for RbpMeasure {
    fn default() -> Self {
        Self::new(0.9, GainsConfig::default(), "")
            .expect("default rbp parameters are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rbp_standard() {
        let measure = RbpMeasure::new(0.9, GainsConfig::default(), "").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. relevance level = 1.
        // gains at rel levels: rel=0 -> gain=0, rel=1 -> gain=1.
        // normalized gains: max=1, min=0 (standard, no change).
        // rank 1: rel=1 -> gain=1. sum += 1 * p^0 = 1.0
        // rank 2: rel=0 -> gain=0. sum += 0
        // rank 3: rel=1 -> gain=1. sum += 1 * p^2 = 0.81
        // total sum = 1.81.
        // result = (1 - 0.9) * 1.81 = 0.1 * 1.81 = 0.181
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.181).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_rbp_custom_p() {
        let measure = RbpMeasure::new(0.5, GainsConfig::default(), "p=0.5").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2.
        // sum = 1 * 0.5^0 + 0 + 1 * 0.5^2 = 1.0 + 0.25 = 1.25.
        // result = (1 - 0.5) * 1.25 = 0.625
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.625)]);
    }

    #[test]
    fn test_rbp_with_per_measure_gains_and_precedence() {
        // Per-measure gains: rel 0 -> 0 (default), rel 1 -> 1, rel 2 -> 4
        // Normalized to [0, 1]: rel 0 -> 0.0, rel 1 -> 0.25, rel 2 -> 1.0
        let measure = RbpMeasure::from_params(Some("p=0.5,1=1,2=4")).unwrap();
        assert_eq!(measure.sub_metrics(), vec!["rbp_p=0.5,1=1,2=4"]);

        let mut config = EvalConfig::default();
        // Global gains would map rel 1 -> 10, rel 2 -> 10, but per-measure gains must win
        config.global_gains = Some(GainsConfig::parse("1=10,2=10"));

        let mut state = make_mock_state(vec![1, 2], 2);
        state.rel_levels = vec![100, 1, 1];
        // rank 1: rel=1 -> norm_gain=0.25, cur_p=1.0 -> 0.25
        // rank 2: rel=2 -> norm_gain=1.0,  cur_p=0.5 -> 0.5
        // val = (1 - 0.5) * (0.25 + 0.5) = 0.375
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.375).abs() < 1e-12);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_rbp_params_validation() {
        let default_m = RbpMeasure::from_params(None).unwrap();
        assert_eq!(default_m.sub_metrics(), vec!["rbp"]);
        assert!(RbpMeasure::from_params(Some("p=0.0")).is_ok());
        assert!(RbpMeasure::from_params(Some("1=2,2=5")).is_ok());

        // Rejects invalid p values
        assert!(RbpMeasure::new(f64::NAN, GainsConfig::default(), "p=nan").is_err());
        assert!(RbpMeasure::new(f64::INFINITY, GainsConfig::default(), "p=inf").is_err());
        assert!(RbpMeasure::new(f64::NEG_INFINITY, GainsConfig::default(), "p=-inf").is_err());
        assert!(RbpMeasure::new(-0.1, GainsConfig::default(), "p=-0.1").is_err());
        assert!(RbpMeasure::new(1.0, GainsConfig::default(), "p=1.0").is_err());
        assert!(RbpMeasure::new(1.5, GainsConfig::default(), "p=1.5").is_err());

        // Rejects malformed, unknown, non-finite, and duplicate params
        assert!(RbpMeasure::from_params(Some("")).is_err());
        assert!(RbpMeasure::from_params(Some("0.5")).is_err());
        assert!(RbpMeasure::from_params(Some("q=1")).is_err());
        assert!(RbpMeasure::from_params(Some("p=0.5,p=0.8")).is_err());
        assert!(RbpMeasure::from_params(Some("1=2,1=5")).is_err());
        assert!(RbpMeasure::from_params(Some("1=2,01=3")).is_err());
        assert!(RbpMeasure::from_params(Some("1=bad")).is_err());
        assert!(RbpMeasure::from_params(Some("1=nan")).is_err());
        assert!(RbpMeasure::from_params(Some("1=inf")).is_err());
        assert!(RbpMeasure::from_params(Some("p=1.0")).is_err());
    }
}
