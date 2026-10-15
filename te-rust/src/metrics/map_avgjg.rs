use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct MapAvgjgMeasure;

impl MapAvgjgMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for MapAvgjgMeasure {
    fn name(&self) -> &'static str {
        "map_avgjg"
    }

    fn short_description(&self) -> &'static str {
        "Mean Average Precision over judgment groups"
    }

    fn explanation(&self) -> &'static str {
        "Mean Average Precision over judgment groups. Precision measured after each relevant doc is retrieved, then averaged for the topic, and then averaged over judgment group (user) and then averaged over topics (if more than one)."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::JudgmentGroups
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["map_avgjg".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let jg_states = match state {
            EvalState::JudgmentGroups(jgs) => jgs.as_slice(),
            EvalState::Standard(q_state) => std::slice::from_ref(q_state),
        };

        if jg_states.is_empty() {
            return vec![MetricValue::Float(0.0)];
        }

        let mut sum_ap = 0.0;
        for q_state in jg_states {
            let mut rel_so_far = 0;
            let mut sum_p = 0.0;

            for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                if rel >= config.relevance_level {
                    rel_so_far += 1;
                    sum_p += (rel_so_far as f64) / (i + 1) as f64;
                }
            }

            if q_state.num_rel > 0 && rel_so_far > 0 {
                sum_ap += sum_p / (q_state.num_rel as f64);
            }
        }

        let final_val = sum_ap / (jg_states.len() as f64);
        vec![MetricValue::Float(final_val)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::QueryEvalState;

    #[test]
    fn test_map_avgjg_standard() {
        let measure = MapAvgjgMeasure::new();
        let config = EvalConfig::default();

        let state1 = QueryEvalState {
            qid: "301".to_string(),
            run_id: "run".to_string(),
            results_rel_list: vec![1, 0, 1],
            num_ret: 3,
            num_rel: 2,
            num_rel_ret: 2,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![1, 2],
        };

        let state2 = QueryEvalState {
            qid: "301".to_string(),
            run_id: "run".to_string(),
            results_rel_list: vec![0, 1, 0],
            num_ret: 3,
            num_rel: 1,
            num_rel_ret: 1,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![2, 1],
        };

        // AP1 = (1/1 + 2/3) / 2 = 5/6 ≈ 0.8333
        // AP2 = (1/2) / 1 = 0.5
        // Mean = (5/6 + 1/2) / 2 = 8/12 = 2/3 ≈ 0.6667
        let scores = measure.calc(&config, &EvalState::JudgmentGroups(vec![state1, state2]));
        assert_eq!(scores.len(), 1);
        if let MetricValue::Float(val) = scores[0] {
            assert!((val - 2.0 / 3.0).abs() < 1e-6);
        } else {
            panic!("Expected Float MetricValue");
        }
    }
}
