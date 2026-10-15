use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RprecMeasure;

impl RprecMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for RprecMeasure {
    fn name(&self) -> &'static str {
        "Rprec"
    }

    fn short_description(&self) -> &'static str {
        "R-Precision"
    }

    fn explanation(&self) -> &'static str {
        "R-Precision. The precision evaluated exactly at rank R, where R is the total number of known relevant documents for the query. This measure assesses system precision at a depth matching the size of the relevance pool."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["Rprec".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let r = q_state.num_rel;
            let mut rel_ret_at_r = 0;

            let limit = std::cmp::min(r, q_state.results_rel_list.len());
            for i in 0..limit {
                if q_state.results_rel_list[i] >= config.relevance_level {
                    rel_ret_at_r += 1;
                }
            }

            let score = if r > 0 {
                (rel_ret_at_r as f64) / (r as f64)
            } else {
                0.0
            };

            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for RprecMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rprec_standard() {
        let measure = RprecMeasure::new();
        let config = EvalConfig::default();
        // r = 2. retrieved = [1, 0, 1]. In top 2: 1 is relevant. score = 1 / 2 = 0.5.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.5)]);
    }
}
