//! `ndcg`, `dcg` and `ideal_dcg`, which trec_eval defines together in `m_ndcg.c`. All three
//! take the same gain map and share one computation of the two sums.

use crate::eval::QueryEvalState;
use crate::metrics::invariants::{self, Invariant};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig, MeasureParseError};

pub struct NdcgMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgMeasure {
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        let name = if params_str.is_empty() {
            "ndcg".to_string()
        } else {
            format!("ndcg_{}", params_str)
        };
        Ok(Self {
            gains_config: GainsConfig::parse(params_str)?,
            sub_metrics: vec![name],
        })
    }
}

/// The DCG of the ranking and the DCG of the ideal ranking, as `te_calc_ndcg` in trec_eval's
/// `m_ndcg.c` computes them. The measure's own gain map comes first, then the global gains,
/// then the default gain, the relevance level. The ranking's sum covers every retrieved
/// document, whatever its gain. The ideal sum covers the judged documents with a positive
/// gain, in descending order of gain.
fn dcg_sums(own_gains: &GainsConfig, config: &EvalConfig, q_state: &QueryEvalState) -> (f64, f64) {
    let gains_config = if !own_gains.custom_gains.is_empty() {
        own_gains
    } else if let Some(ref gg) = config.global_gains {
        gg
    } else {
        own_gains
    };
    let gains = Gains::setup(gains_config, &q_state.rel_levels);

    let mut results_dcg = 0.0;
    let mut ideal_dcg = 0.0;
    let mut cur_level = gains.rel_gains.len() as i64 - 1;
    let mut ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
    let mut num_at_level = 0;

    let mut i = 0;
    while i < q_state.num_ret && ideal_gain > 0.0 {
        let results_gain = gains.get_gain(q_state.results_rel_list[i]);
        if results_gain != 0.0 {
            results_dcg += results_gain / ((i + 2) as f64).log2();
        }

        num_at_level += 1;
        while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
            num_at_level = 1;
            cur_level -= 1;
            ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
        }

        if ideal_gain > 0.0 {
            ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
        }

        i += 1;
    }

    while i < q_state.num_ret {
        let results_gain = gains.get_gain(q_state.results_rel_list[i]);
        if results_gain != 0.0 {
            results_dcg += results_gain / ((i + 2) as f64).log2();
        }
        i += 1;
    }

    while ideal_gain > 0.0 {
        num_at_level += 1;
        while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
            num_at_level = 1;
            cur_level -= 1;
            ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
        }
        if ideal_gain > 0.0 {
            ideal_dcg += ideal_gain / ((i + 2) as f64).log2();
        }
        i += 1;
    }

    (results_dcg, ideal_dcg)
}

impl Measure for NdcgMeasure {
    fn name(&self) -> &'static str {
        "ndcg"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain. Computes NDCG over the entire retrieved ranking and entire ideal ranking."
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
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let (results_dcg, ideal_dcg) = dcg_sums(&self.gains_config, config, q_state);
        let score = if ideal_dcg > 0.0 { results_dcg / ideal_dcg } else { 0.0 };
        vec![MetricValue::Float(score)]
    }

}

impl Default for NdcgMeasure {
    fn default() -> Self {
        Self::new("").expect("an empty gain map is valid")
    }
}

/// Discounted cumulative gain of the ranking, the numerator of `ndcg`, with the same gains.
pub struct DcgMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl DcgMeasure {
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        let name = if params_str.is_empty() {
            "dcg".to_string()
        } else {
            format!("dcg_{}", params_str)
        };
        Ok(Self {
            gains_config: GainsConfig::parse(params_str)?,
            sub_metrics: vec![name],
        })
    }
}

impl Measure for DcgMeasure {
    fn name(&self) -> &'static str {
        "dcg"
    }

    fn short_description(&self) -> &'static str {
        "Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Discounted Cumulative Gain. Sums gain / log2(rank + 1) over the entire retrieved ranking, with the gains of ndcg. ndcg is dcg / ideal_dcg."
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

    fn invariants(&self) -> &'static [Invariant] {
        invariants::STANDARD
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };
        let (results_dcg, _) = dcg_sums(&self.gains_config, config, q_state);
        vec![MetricValue::Float(results_dcg)]
    }
}

impl Default for DcgMeasure {
    fn default() -> Self {
        Self::new("").expect("an empty gain map is valid")
    }
}

/// The ideal ranking's sum depends on the judgments alone, and an empty ranking does not
/// make it 0.
const IDEAL_DCG_INVARIANTS: &[Invariant] = &[
    Invariant::Finite,
    Invariant::NonNegative,
    Invariant::ZeroRelevanceIsZero,
];

/// Discounted cumulative gain of the ideal ranking, the denominator of `ndcg`, with the
/// same gains.
pub struct IdealDcgMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl IdealDcgMeasure {
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        let name = if params_str.is_empty() {
            "ideal_dcg".to_string()
        } else {
            format!("ideal_dcg_{}", params_str)
        };
        Ok(Self {
            gains_config: GainsConfig::parse(params_str)?,
            sub_metrics: vec![name],
        })
    }
}

impl Measure for IdealDcgMeasure {
    fn name(&self) -> &'static str {
        "ideal_dcg"
    }

    fn short_description(&self) -> &'static str {
        "Ideal Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Ideal Discounted Cumulative Gain. The DCG of the judged documents with a positive gain, in descending order of gain, with the gains of ndcg. ndcg is dcg / ideal_dcg."
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

    fn invariants(&self) -> &'static [Invariant] {
        IDEAL_DCG_INVARIANTS
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };
        let (_, ideal_dcg) = dcg_sums(&self.gains_config, config, q_state);
        vec![MetricValue::Float(ideal_dcg)]
    }
}

impl Default for IdealDcgMeasure {
    fn default() -> Self {
        Self::new("").expect("an empty gain map is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_standard() {
        let measure = NdcgMeasure::new("").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // dcg = 1.0 / log2(2) + 0.0 + 1.0 / log2(4) = 1.0 + 0.5 = 1.5.
        // idcg = 1.0 / log2(2) + 1.0 / log2(3) = 1.0 + 1.0 / 1.58496 = 1.63092975
        // ndcg = 1.5 / 1.63092975 = 0.91972079
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.919720791448).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn dcg_and_ideal_dcg_are_the_two_sums_of_ndcg() {
        // The ranking of test_ndcg_standard: dcg = 1.5 and ideal_dcg = 1 + 1/log2(3).
        let config = EvalConfig::default();
        let state = EvalState::Standard(make_mock_state(vec![1, 0, 1], 2));
        let value = |values: Vec<MetricValue>| match values[..] {
            [MetricValue::Float(v)] => v,
            _ => panic!("Expected one float value"),
        };
        let dcg = value(DcgMeasure::new("").unwrap().calc(&config, &state));
        let ideal = value(IdealDcgMeasure::new("").unwrap().calc(&config, &state));
        let ndcg = value(NdcgMeasure::new("").unwrap().calc(&config, &state));
        assert!((dcg - 1.5).abs() < 1e-12);
        assert!((ideal - (1.0 + 1.0 / 3f64.log2())).abs() < 1e-12);
        assert_eq!(ndcg, dcg / ideal);
    }
}
