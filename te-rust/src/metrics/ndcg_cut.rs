use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig};

pub struct NdcgCutMeasure {
    cutoffs: Vec<usize>,
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("ndcg_cut_{}", c)).collect();
        Self {
            cutoffs,
            gains_config: GainsConfig::parse(""),
            sub_metrics,
        }
    }
}

impl Measure for NdcgCutMeasure {
    fn name(&self) -> &'static str {
        "ndcg_cut"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain at cutoff ranks. Compares the discounted cumulative gain of the retrieved ranking against that of the ideal ranking of known relevant documents, evaluated at specified rank thresholds. NDCG accounts for graded relevance judgments and discounts items at lower ranks. See Cumulated gain-based evaluation of IR techniques by Jarvelin and Kekalainen (2002, ACM TOIS, doi:10.1145/582415.582418) for the formal definition."
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
        vec![MetricValue::Float(0.0); self.cutoffs.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let gains_config = if !self.gains_config.custom_gains.is_empty() {
            &self.gains_config
        } else if let Some(ref gg) = config.global_gains {
            gg
        } else {
            &self.gains_config
        };
        let gains = Gains::setup(gains_config, &q_state.rel_levels);
        let num_params = self.cutoffs.len();
        let mut dcgs = vec![0.0; num_params];
        let mut idcgs = vec![0.0; num_params];


                // 1. Calculate DCG at each cutoff
                let mut cutoff_index = 0;
                let mut dcg_sum = 0.0;

                for i in 0..q_state.num_ret {
                    if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                        dcgs[cutoff_index] = dcg_sum;
                        cutoff_index += 1;
                        while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                            dcgs[cutoff_index] = dcg_sum;
                            cutoff_index += 1;
                        }
                        if cutoff_index >= num_params {
                            break;
                        }
                    }
                    let rel = q_state.results_rel_list[i];
                    let results_gain = gains.get_gain(rel);
                    if results_gain != 0.0 {
                        dcg_sum += results_gain / ((i + 2) as f64).log2();
                    }
                }
                while cutoff_index < num_params {
                    dcgs[cutoff_index] = dcg_sum;
                    cutoff_index += 1;
                }

                // 2. Calculate Ideal DCG (IDCG) at each cutoff
                cutoff_index = 0;
                let mut cur_lvl_idx = gains.rel_gains.len() as i64 - 1;
                let mut lvl_count = 0;
                let mut ideal_dcg_sum = 0.0;

                let mut i = 0;
                loop {
                    lvl_count += 1;
                    while cur_lvl_idx >= 0 && lvl_count > gains.rel_gains[cur_lvl_idx as usize].num_at_level {
                        cur_lvl_idx -= 1;
                        lvl_count = 1;
                    }
                    let ideal_gain = if cur_lvl_idx >= 0 { gains.rel_gains[cur_lvl_idx as usize].gain } else { 0.0 };
                    if ideal_gain <= 0.0 {
                        break;
                    }

                    if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                        idcgs[cutoff_index] = ideal_dcg_sum;
                        cutoff_index += 1;
                        while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                            idcgs[cutoff_index] = ideal_dcg_sum;
                            cutoff_index += 1;
                        }
                        if cutoff_index >= num_params {
                            break;
                        }
                    }

                    ideal_dcg_sum += ideal_gain / ((i + 2) as f64).log2();
                    i += 1;
                }
                while cutoff_index < num_params {
                    idcgs[cutoff_index] = ideal_dcg_sum;
                    cutoff_index += 1;
                }

                // 3. Normalize DCG by IDCG
                let mut results = Vec::with_capacity(num_params);
                for k in 0..num_params {
                    let idcg = idcgs[k];
                    let dcg = dcgs[k];
                    let score = if idcg > 0.0 { dcg / idcg } else { 0.0 };
                    results.push(MetricValue::Float(score));
                }

                results
    }
}


impl Default for NdcgCutMeasure {
    fn default() -> Self {
        Self::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_cut_standard() {
        let measure = NdcgCutMeasure::new(vec![1, 3]);
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // dcg@1 = 1.0. dcg@3 = 1.0 + 0.0 + 1.0 / log2(4) = 1.5.
        // idcg@1 = 1.0. idcg@3 = 1.0 + 1.0 / log2(3) = 1.0 + 1.0 / 1.5849625 = 1.63092975.
        // ndcg@1 = 1.0. ndcg@3 = 1.5 / 1.63092975 = 0.91972079.
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0], MetricValue::Float(1.0));
        if let MetricValue::Float(v) = actual[1] {
            assert!((v - 0.919720791448).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_ndcg_cut_global_gains() {
        let measure = NdcgCutMeasure::new(vec![1, 3]);
        let mut config = EvalConfig::default();
        config.global_gains = Some(GainsConfig::parse("1=5.0,2=10.0"));
        
        let state = crate::eval::alignment::QueryEvalState {
            qid: "mock_topic".to_string(),
            run_id: "mock_run".to_string(),
            results_rel_list: vec![1, 0, 2],
            num_ret: 3,
            num_rel: 2,
            num_rel_ret: 2,
            num_nonpool: 0,
            num_unjudged_in_pool: 0,
            rel_levels: vec![100, 1, 1],
        };
        
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0], MetricValue::Float(0.5));
        if let MetricValue::Float(v) = actual[1] {
            assert!((v - 0.7601875).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
