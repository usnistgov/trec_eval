use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct IprecAtRecallMeasure {
    cutoff_percents: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl IprecAtRecallMeasure {
    pub fn new(cutoff_percents: Vec<f64>) -> Self {
        let sub_metrics = cutoff_percents.iter().map(|p| format!("iprec_at_recall_{:.2}", p)).collect();
        Self { cutoff_percents, sub_metrics }
    }
}

impl Measure for IprecAtRecallMeasure {
    fn name(&self) -> &'static str {
        "iprec_at_recall"
    }

    fn short_description(&self) -> &'static str {
        "Interpolated Precision at recall cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Interpolated Precision at specified recall thresholds. Commonly used to plot the standard Recall-Precision graph. Precision interpolation at recall level X is the maximum precision achieved at any rank from that point onwards: Int_Prec (rankX) == MAX (Prec (rankY)) for all Y >= X."
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

        // 1. Translate percentage of rels to actual cutoff counts of relevant documents using lround/round
        let cutoffs: Vec<i64> = self.cutoff_percents
            .iter()
            .map(|&p| (p * q_state.num_rel as f64).round() as i64)
            .collect();

        let mut current_cut = (num_params as i64) - 1;

        // 2. Adjust starting cutoff point if we didn't retrieve enough relevant documents
        while current_cut >= 0 && cutoffs[current_cut as usize] > (q_state.num_rel_ret as i64) {
            current_cut -= 1;
        }

        // 3. Loop in reverse order to calculate the online MAX (interpolated) precision
        let mut precis = if q_state.num_ret > 0 {
            (q_state.num_rel_ret as f64) / (q_state.num_ret as f64)
        } else {
            0.0
        };
        let mut int_precis = precis;
        let mut rel_so_far = q_state.num_rel_ret;

        for i in (1..=q_state.num_ret).rev() {
            precis = (rel_so_far as f64) / (i as f64);
            if precis > int_precis {
                int_precis = precis;
            }
            if q_state.results_rel_list[i - 1] >= config.relevance_level {
                while current_cut >= 0 && (rel_so_far as i64) == cutoffs[current_cut as usize] {
                    results[current_cut as usize] = int_precis;
                    current_cut -= 1;
                }
                if rel_so_far > 0 {
                    rel_so_far -= 1;
                }
            }
        }

        // 4. Fill in any remaining unreached cutoffs (such as recall 0.0)
        while current_cut >= 0 {
            results[current_cut as usize] = int_precis;
            current_cut -= 1;
        }

        results.into_iter().map(MetricValue::Float).collect()
    }

}

impl Default for IprecAtRecallMeasure {
    fn default() -> Self {
        Self::new(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_iprec_at_recall_standard() {
        let measure = IprecAtRecallMeasure::new(vec![0.0, 0.5, 1.0]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2, num_ret = 3, num_rel_ret = 2.
        // percents: [0.0, 0.5, 1.0]
        // cutoffs:
        //   0.0 * 2 = 0
        //   0.5 * 2 = 1
        //   1.0 * 2 = 2
        //
        // Reverse trace:
        // start: precis = 2/3 = 0.666667. int_precis = 0.666667. rel_so_far = 2. current_cut = 2 (cutoff = 2).
        // i = 3: precis = 2/3 = 0.666667.
        //   doc at index 2 (value 1) is relevant:
        //   while rel_so_far (2) == cutoffs[current_cut] (2):
        //     results[2] = int_precis (0.666667)
        //     current_cut = 1 (cutoff = 1)
        //   rel_so_far becomes 1.
        // i = 2: precis = 1/2 = 0.5.
        //   doc at index 1 is 0.
        // i = 1: precis = 1/1 = 1.0. int_precis becomes 1.0.
        //   doc at index 0 (value 1) is relevant:
        //   while rel_so_far (1) == cutoffs[1] (1):
        //     results[1] = int_precis (1.0)
        //     current_cut = 0 (cutoff = 0)
        //   rel_so_far becomes 0.
        // end loop:
        // while current_cut (0) >= 0:
        //   results[0] = int_precis (1.0)
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0], MetricValue::Float(1.0));
        assert_eq!(actual[1], MetricValue::Float(1.0));
        if let MetricValue::Float(v) = actual[2] {
            assert!((v - 0.66666667).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
