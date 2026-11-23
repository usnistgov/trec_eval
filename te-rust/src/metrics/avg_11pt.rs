use crate::metrics::common::MeasureParseError;
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct Avg11PtMeasure {
    cutoffs: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl Avg11PtMeasure {
    pub fn new(mut cutoffs: Vec<f64>, params_str: &str) -> Result<Self, MeasureParseError> {
        if cutoffs.is_empty() {
            return Err(MeasureParseError::new("recall level list must not be empty"));
        }
        for &v in &cutoffs {
            if !v.is_finite() {
                return Err(MeasureParseError::new(format!("non-finite recall level '{}'", v)));
            }
            if !(0.0..=1.0).contains(&v) {
                return Err(MeasureParseError::new(format!(
                    "recall level '{}' must be between 0.0 and 1.0",
                    v
                )));
            }
        }
        cutoffs.sort_by(f64::total_cmp);
        for w in cutoffs.windows(2) {
            if w[0] == w[1] {
                return Err(MeasureParseError::new(format!(
                    "duplicate recall level '{}'",
                    w[0]
                )));
            }
        }
        let name = if params_str.is_empty() {
            "11pt_avg".to_string()
        } else {
            format!("11pt_avg_{}", params_str)
        };
        Ok(Self { cutoffs, sub_metrics: vec![name] })
    }
}

impl Measure for Avg11PtMeasure {
    fn name(&self) -> &'static str {
        "11pt_avg"
    }

    fn short_description(&self) -> &'static str {
        "11pt Interpolated Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Interpolated Precision averaged over 11 recall points. Calculates interpolated precision scores at 11 standard recall levels (0.0, 0.1, 0.2, ..., 1.0) and averages them."
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

        let num_rel = q_state.num_rel;
        let num_ret = q_state.results_rel_list.len();

        if num_rel == 0 {
            return vec![MetricValue::Float(0.0)];
        }

        // Count total relevant retrieved
        let mut num_rel_ret = 0;
        for &rel in &q_state.results_rel_list {
            if rel >= config.relevance_level {
                num_rel_ret += 1;
            }
        }

        // Translate cutoff percentages to target relevant document counts
        let num_params = self.cutoffs.len();
        let mut cutoffs = vec![0i64; num_params];
        for i in 0..num_params {
            cutoffs[i] = (self.cutoffs[i] * num_rel as f64).round() as i64;
        }

        let mut current_cut = num_params as i64 - 1;
        while current_cut >= 0 && cutoffs[current_cut as usize] > num_rel_ret as i64 {
            current_cut -= 1;
        }

        let mut sum = 0.0;
        let mut int_precis = if num_ret > 0 {
            (num_rel_ret as f64) / (num_ret as f64)
        } else {
            0.0
        };
        let mut rel_so_far = num_rel_ret;

        for i in (1..=num_ret).rev() {
            let precis = (rel_so_far as f64) / (i as f64);
            if int_precis < precis {
                int_precis = precis;
            }
            if q_state.results_rel_list[i - 1] >= config.relevance_level {
                while current_cut >= 0 && rel_so_far as i64 == cutoffs[current_cut as usize] {
                    sum += int_precis;
                    current_cut -= 1;
                }
                rel_so_far -= 1;
            }
        }

        while current_cut >= 0 {
            sum += int_precis;
            current_cut -= 1;
        }

        let final_val = sum / (num_params as f64);
        vec![MetricValue::Float(final_val)]
    }
}

impl Default for Avg11PtMeasure {
    fn default() -> Self {
        Self::new(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0], "")
            .expect("default 11pt_avg recall levels are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_avg_11pt_standard() {
        let measure = Avg11PtMeasure::default();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
        if let MetricValue::Float(v) = actual[0] {
            assert!(v >= 0.0 && v <= 1.0);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_avg_11pt_unsorted_cutoffs() {
        let config = EvalConfig::default();
        let state = EvalState::Standard(make_mock_state(vec![1, 0, 0, 1], 2));

        let asc = Avg11PtMeasure::new(vec![0.5, 1.0], "0.5,1.0").unwrap();
        let desc = Avg11PtMeasure::new(vec![1.0, 0.5], "1.0,0.5").unwrap();

        assert_eq!(desc.sub_metrics(), vec!["11pt_avg_1.0,0.5"]);
        assert_eq!(asc.calc(&config, &state), vec![MetricValue::Float(0.75)]);
        assert_eq!(desc.calc(&config, &state), vec![MetricValue::Float(0.75)]);
    }

    #[test]
    fn test_avg_11pt_rejects_invalid_cutoffs() {
        assert!(Avg11PtMeasure::new(vec![], "").is_err());
        assert!(Avg11PtMeasure::new(vec![f64::NAN], "nan").is_err());
        assert!(Avg11PtMeasure::new(vec![f64::INFINITY], "inf").is_err());
        assert!(Avg11PtMeasure::new(vec![f64::NEG_INFINITY], "-inf").is_err());
        assert!(Avg11PtMeasure::new(vec![-0.1, 0.5], "-0.1,0.5").is_err());
        assert!(Avg11PtMeasure::new(vec![0.5, 1.5], "0.5,1.5").is_err());
        assert!(Avg11PtMeasure::new(vec![0.5, 0.5], "0.5,0.5").is_err());
        assert!(Avg11PtMeasure::new(vec![-0.0, 0.0], "-0.0,0.0").is_err());
        assert!(Avg11PtMeasure::new(vec![0.0, 1.0], "0.0,1.0").is_ok());
    }
}
