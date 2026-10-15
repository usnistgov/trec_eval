use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RprecMultMeasure {
    cutoff_percents: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl RprecMultMeasure {
    pub fn new(cutoff_percents: Vec<f64>) -> Self {
        let sub_metrics = cutoff_percents.iter().map(|p| format!("Rprec_mult_{:.2}", p)).collect();
        Self { cutoff_percents, sub_metrics }
    }
}

impl Measure for RprecMultMeasure {
    fn name(&self) -> &'static str {
        "Rprec_mult"
    }

    fn short_description(&self) -> &'static str {
        "Precision measured at multiples of R (num_rel)"
    }

    fn explanation(&self) -> &'static str {
        "Precision measured at specified multiples of the relevance pool size R. Useful to determine whether methods are precision-oriented or recall-oriented."
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
        vec![MetricValue::Float(0.0); self.cutoff_percents.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let num_params = self.cutoff_percents.len();
        let mut results = vec![0.0; num_params];

        if q_state.num_rel == 0 {
            return results.into_iter().map(MetricValue::Float).collect();
        }

        // 1. Convert percentages to integer rank cutoffs using C's + 0.9 cast rounding
        let cutoffs: Vec<i64> = self.cutoff_percents
            .iter()
            .map(|&p| (p * q_state.num_rel as f64 + 0.9) as i64)
            .collect();

        let mut current_cut = (num_params as i64) - 1;

        // 2. Pre-fill any cutoffs that exceed the retrieved length
        while current_cut >= 0 && cutoffs[current_cut as usize] > (q_state.num_ret as i64) {
            let cutoff = cutoffs[current_cut as usize];
            results[current_cut as usize] = if cutoff > 0 {
                (q_state.num_rel_ret as f64) / (cutoff as f64)
            } else {
                0.0
            };
            current_cut -= 1;
        }

        // 3. Scan the retrieved documents backwards to compute precision at exact cutoffs
        let mut rel_so_far = q_state.num_rel_ret;
        for i in (1..=q_state.num_ret).rev() {
            let precis = (rel_so_far as f64) / (i as f64);
            while current_cut >= 0 && (i as i64) == cutoffs[current_cut as usize] {
                results[current_cut as usize] = precis;
                current_cut -= 1;
            }
            if q_state.results_rel_list[i - 1] >= config.relevance_level {
                if rel_so_far > 0 {
                    rel_so_far -= 1;
                }
            }
        }

        results.into_iter().map(MetricValue::Float).collect()
    }

}

impl Default for RprecMultMeasure {
    fn default() -> Self {
        Self::new(vec![0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rprec_mult_standard() {
        let measure = RprecMultMeasure::new(vec![0.5, 1.0, 1.5]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2. num_ret = 3, num_rel_ret = 2.
        // percents: [0.5, 1.0, 1.5]
        // cutoffs:
        //   0.5 * 2 + 0.9 = 1.9 -> 1
        //   1.0 * 2 + 0.9 = 2.9 -> 2
        //   1.5 * 2 + 0.9 = 3.9 -> 3
        // All cutoffs <= num_ret (3).
        // Let's trace reverse scan:
        // i = 3: precis = 2 / 3 = 0.666667.
        //   Is 3 == cutoffs[2] (3)? Yes, results[2] = 0.666667.
        //   results_rel_list[2] is 1 (relevant) -> rel_so_far becomes 1.
        // i = 2: precis = 1 / 2 = 0.5.
        //   Is 2 == cutoffs[1] (2)? Yes, results[1] = 0.5.
        //   results_rel_list[1] is 0 (non-relevant) -> rel_so_far remains 1.
        // i = 1: precis = 1 / 1 = 1.0.
        //   Is 1 == cutoffs[0] (1)? Yes, results[0] = 1.0.
        //   results_rel_list[0] is 1 -> rel_so_far becomes 0.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0], MetricValue::Float(1.0));
        assert_eq!(actual[1], MetricValue::Float(0.5));
        if let MetricValue::Float(v) = actual[2] {
            assert!((v - 0.66666667).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
