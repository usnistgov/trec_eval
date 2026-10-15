use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RprecMultAvgjgMeasure {
    cutoff_percents: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl RprecMultAvgjgMeasure {
    pub fn new(cutoff_percents: Vec<f64>) -> Self {
        let sub_metrics = cutoff_percents
            .iter()
            .map(|p| format!("Rprec_mult_avgjg_{:.2}", p))
            .collect();
        Self {
            cutoff_percents,
            sub_metrics,
        }
    }
}

impl Measure for RprecMultAvgjgMeasure {
    fn name(&self) -> &'static str {
        "Rprec_mult_avgjg"
    }

    fn short_description(&self) -> &'static str {
        "Precision measured at multiples of R averaged over judgment groups"
    }

    fn explanation(&self) -> &'static str {
        "Precision measured at multiples of R(num_rel) averaged over users. If there is more than one judgment group, the measure is averaged over those jgs."
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
        vec![MetricValue::Float(0.0); self.cutoff_percents.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let jg_states = match state {
            EvalState::JudgmentGroups(jgs) => jgs.as_slice(),
            EvalState::Standard(q_state) => std::slice::from_ref(q_state),
        };

        let num_params = self.cutoff_percents.len();
        if jg_states.is_empty() {
            return vec![MetricValue::Float(0.0); num_params];
        }

        let mut total_results = vec![0.0; num_params];

        for q_state in jg_states {
            if q_state.num_rel == 0 {
                continue;
            }

            // Convert percentages to integer rank cutoffs using C's + 0.9 cast rounding
            let cutoffs: Vec<i64> = self
                .cutoff_percents
                .iter()
                .map(|&p| (p * q_state.num_rel as f64 + 0.9) as i64)
                .collect();

            let mut jg_results = vec![0.0; num_params];
            let mut current_cut = (num_params as i64) - 1;

            // Pre-fill cutoffs exceeding retrieved length
            while current_cut >= 0 && cutoffs[current_cut as usize] > (q_state.num_ret as i64) {
                let cutoff = cutoffs[current_cut as usize];
                jg_results[current_cut as usize] = if cutoff > 0 {
                    (q_state.num_rel_ret as f64) / (cutoff as f64)
                } else {
                    0.0
                };
                current_cut -= 1;
            }

            // Scan retrieved documents backwards
            let mut rel_so_far = q_state.num_rel_ret;
            for i in (1..=q_state.num_ret).rev() {
                let precis = (rel_so_far as f64) / (i as f64);
                while current_cut >= 0 && (i as i64) == cutoffs[current_cut as usize] {
                    jg_results[current_cut as usize] = precis;
                    current_cut -= 1;
                }
                if q_state.results_rel_list[i - 1] >= config.relevance_level {
                    rel_so_far -= 1;
                }
            }

            for idx in 0..num_params {
                total_results[idx] += jg_results[idx];
            }
        }

        let num_jgs = jg_states.len() as f64;
        total_results
            .into_iter()
            .map(|val| MetricValue::Float(val / num_jgs))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::QueryEvalState;

    #[test]
    fn test_rprec_mult_avgjg_standard() {
        let measure = RprecMultAvgjgMeasure::new(vec![0.5, 1.0]);
        let config = EvalConfig::default();

        let state1 = QueryEvalState {
            qid: "301".to_string(),
            run_id: "run".to_string(),
            results_rel_list: vec![1, 0, 1, 0],
            num_ret: 4,
            num_rel: 2,
            num_rel_ret: 2,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![2, 2],
        };

        // For state1 (num_rel = 2):
        // cutoffs: 0.5*2+0.9 = 1.9 -> 1; 1.0*2+0.9 = 2.9 -> 2
        // Rprec_0.5 = P@1 = 1.0; Rprec_1.0 = P@2 = 1/2 = 0.5
        let scores = measure.calc(&config, &EvalState::JudgmentGroups(vec![state1]));
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0], MetricValue::Float(1.0));
        assert_eq!(scores[1], MetricValue::Float(0.5));
    }
}
