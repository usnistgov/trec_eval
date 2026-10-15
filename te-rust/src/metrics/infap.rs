use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct InfAPMeasure;

impl InfAPMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for InfAPMeasure {
    fn name(&self) -> &'static str {
        "infAP"
    }

    fn short_description(&self) -> &'static str {
        "Inferred Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Inferred Average Precision. A measure that estimates Average Precision when relevance judgments are incomplete or sampled. Assumes a judgment pool where a random subset of documents is judged. It evaluates precision at each judged relevant document by estimating the proportion of relevant documents among the higher retrieved ones, corrected for unpooled documents."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["infAP".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let mut nonrel_so_far = 0;
        let mut rel_so_far = 0;
        let mut pool_unjudged_so_far = 0;
        let mut inf_ap = 0.0;
        let epsilon = 0.00001;

        for (j, &rel) in q_state.results_rel_list.iter().enumerate() {
            if rel == crate::eval::alignment::RELVALUE_NONPOOL {
                continue;
            }
            if rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                pool_unjudged_so_far += 1;
                continue;
            }

            if rel >= 0 && rel < config.relevance_level {
                nonrel_so_far += 1;
            } else {
                rel_so_far += 1;
                if j == 0 {
                    inf_ap += 1.0;
                } else {
                    let fj = j as f64;
                    let term1 = 1.0 / (fj + 1.0);
                    let term2 = fj / (fj + 1.0);
                    let numer_p = (rel_so_far - 1 + nonrel_so_far + pool_unjudged_so_far) as f64;
                    let ratio_p = numer_p / fj;
                    let rel_term = (rel_so_far - 1) as f64;
                    let nonrel_term = nonrel_so_far as f64;
                    let ratio_rel = (rel_term + epsilon) / (rel_term + nonrel_term + 2.0 * epsilon);
                    inf_ap += term1 + term2 * ratio_p * ratio_rel;
                }
            }
        }

        if q_state.num_rel > 0 {
            inf_ap /= q_state.num_rel as f64;
        } else {
            inf_ap = 0.0;
        }

        vec![MetricValue::Float(inf_ap)]
    }
}

impl Default for InfAPMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_infap_standard() {
        let measure = InfAPMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            // j = 0: rel_so_far = 1, inf_ap += 1.0
            // j = 1: nonrel_so_far = 1, rel = 0
            // j = 2: rel_so_far = 2
            //   fj = 2.0
            //   term1 = 1/3 = 0.3333333
            //   term2 = 2/3 = 0.6666667
            //   numer_p = (2 - 1 + 1 + 0) = 2.0
            //   ratio_p = 2.0 / 2.0 = 1.0
            //   rel_term = 1.0
            //   nonrel_term = 1.0
            //   ratio_rel = (1.0 + 1e-5) / (1.0 + 1.0 + 2e-5) = 1.00001 / 2.00002 = 0.5
            //   inf_ap += 1/3 + 2/3 * 1.0 * 0.5 = 1/3 + 1/3 = 2/3 = 0.6666667
            // Total sum = 1.6666667
            // inf_ap = 1.6666667 / 2 = 0.8333333
            assert!((v - 0.83333333).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
