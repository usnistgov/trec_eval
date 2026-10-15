use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct SetMapMeasure;

impl SetMapMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for SetMapMeasure {
    fn name(&self) -> &'static str {
        "set_map"
    }

    fn short_description(&self) -> &'static str {
        "Set Mean Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Set map: num_relevant_retrieved**2 / (num_retrieved*num_rel)\n\
        Unranked set map, where the precision due to all relevant retrieved docs\n\
        is the set precision, and the precision due to all relevant not-retrieved\n\
        docs is set to 0."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["set_map".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let score = if q_state.num_ret > 0 && q_state.num_rel > 0 {
                let rel_ret = q_state.num_rel_ret as f64;
                (rel_ret * rel_ret) / ((q_state.num_ret as f64) * (q_state.num_rel as f64))
            } else {
                0.0
            };
            vec![MetricValue::Float(score)]
        } else {
            self.initial_values()
        }
    }
}

impl Default for SetMapMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_set_map_standard() {
        let measure = SetMapMeasure::new();
        let config = EvalConfig::default();
        // retrieved: 3, relevant: 2, relevant retrieved: 2.
        // score: (2 * 2) / (3 * 2) = 4 / 6 = 2/3.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(2.0 / 3.0)]);
    }
}
