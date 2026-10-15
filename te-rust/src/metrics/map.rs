use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct MapMeasure;

impl MapMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for MapMeasure {
    fn name(&self) -> &'static str {
        "map"
    }

    fn short_description(&self) -> &'static str {
        "Mean Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Mean Average Precision. Calculates the average of precision scores evaluated at each rank where a relevant document is retrieved. For documents not retrieved, precision is defined as 0. Over queried topics, MAP represents the mean of average precisions."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["map".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        if let Some(q_state) = state.get_standard() {
            let mut rel_so_far = 0;
            let mut sum = 0.0;

            for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                if rel >= config.relevance_level {
                    rel_so_far += 1;
                    sum += (rel_so_far as f64) / (i + 1) as f64;
                }
            }

            let ap = if q_state.num_rel > 0 {
                sum / (q_state.num_rel as f64)
            } else {
                0.0
            };

            vec![MetricValue::Float(ap)]
        } else {
            self.initial_values()
        }
    }

}

impl Default for MapMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_map_standard() {
        let measure = MapMeasure::new();
        let config = EvalConfig::default();
        // Rank 1: rel (P@1=1.0), Rank 2: nonrel (P@2=0.5), Rank 3: rel (P@3=2/3=0.6667). num_rel=2.
        // sum = 1.0 + 2/3 = 1.6667. ap = 1.6667 / 2 = 0.8333
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.8333333333333333).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_map_all_relevant_at_end() {
        let measure = MapMeasure::new();
        let config = EvalConfig::default();
        // Rank 3 is relevant (P@3 = 1/3 = 0.3333). num_rel = 1.
        let state = make_mock_state(vec![0, 0, 1], 1);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.3333333333333333).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }
}
