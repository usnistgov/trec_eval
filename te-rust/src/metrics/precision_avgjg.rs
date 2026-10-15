use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct PrecisionAvgjgMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl PrecisionAvgjgMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("P_avgjg_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for PrecisionAvgjgMeasure {
    fn name(&self) -> &'static str {
        "P_avgjg"
    }

    fn short_description(&self) -> &'static str {
        "Precision at cutoffs, averaged over judgment groups"
    }

    fn explanation(&self) -> &'static str {
        "Precision at cutoffs, averaged over judgment groups (users). Precision measured at various doc level cutoffs in the ranking, averaged over the judgment groups."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::JudgmentGroups
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0); self.cutoffs.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let jg_states = match state {
            EvalState::JudgmentGroups(jgs) => jgs.as_slice(),
            EvalState::Standard(q_state) => std::slice::from_ref(q_state),
        };

        if jg_states.is_empty() {
            return vec![MetricValue::Float(0.0); self.cutoffs.len()];
        }

        let mut results = vec![0.0; self.cutoffs.len()];

        for q_state in jg_states {
            for (idx, &c) in self.cutoffs.iter().enumerate() {
                let rel_ret = super::common::count_relevant_retrieved_up_to(&q_state.results_rel_list, c, config);
                let p = if c > 0 {
                    (rel_ret as f64) / (c as f64)
                } else {
                    0.0
                };
                results[idx] += p;
            }
        }

        let num_jgs = jg_states.len() as f64;
        results
            .into_iter()
            .map(|sum_p| MetricValue::Float(sum_p / num_jgs))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::QueryEvalState;

    #[test]
    fn test_precision_avgjg_standard() {
        let measure = PrecisionAvgjgMeasure::new(vec![5, 10]);
        let config = EvalConfig::default();

        let state1 = QueryEvalState {
            qid: "301".to_string(),
            run_id: "run".to_string(),
            results_rel_list: vec![1, 1, 0, 0, 0],
            num_ret: 5,
            num_rel: 2,
            num_rel_ret: 2,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![3, 2],
        };

        let state2 = QueryEvalState {
            qid: "301".to_string(),
            run_id: "run".to_string(),
            results_rel_list: vec![0, 0, 0, 0, 0],
            num_ret: 5,
            num_rel: 2,
            num_rel_ret: 0,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![3, 2],
        };

        // P@5 for state1 = 2/5 = 0.4; state2 = 0/5 = 0.0 -> Mean = 0.2
        // P@10 for state1 = 2/10 = 0.2; state2 = 0/10 = 0.0 -> Mean = 0.1
        let scores = measure.calc(&config, &EvalState::JudgmentGroups(vec![state1, state2]));
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0], MetricValue::Float(0.2));
        assert_eq!(scores[1], MetricValue::Float(0.1));
    }
}
