pub mod common;
pub mod runid;
pub mod num_q;
pub mod num_ret;
pub mod num_rel;
pub mod num_rel_ret;
pub mod map;
pub mod rprec;
pub mod recip_rank;
pub mod bpref;
pub mod precision;
pub mod ndcg_cut;
pub mod ndcg;
pub mod recall;
pub mod success;
pub mod avg_11pt;
pub mod utility;
pub mod relstring;
pub mod set_p;
pub mod set_recall;
pub mod set_relative_p;
pub mod set_map;
pub mod set_f;
pub mod g;
pub mod map_cut;
pub mod relative_p;
pub mod rprec_mult;
pub mod iprec_at_recall;
pub mod gm_map;
pub mod gm_bpref;
pub mod infap;
pub mod unj;
pub mod num_nonrel_judged_ret;
pub mod rbp;
pub mod rbp_resid;
pub mod yaap;
pub mod bin_g;
pub mod ndcg_rel;
pub mod rndcg;
pub mod ndcg_p;
pub mod map_avgjg;
pub mod precision_avgjg;
pub mod rprec_mult_avgjg;
pub mod registry;
pub mod invariants;


use crate::eval::QueryEvalState;
use crate::metrics::invariants::Invariant;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueFormat {
    /// Render as a double-precision float with exactly 4 decimal places (e.g. 0.2543).
    Float,
    /// Render as an integer (e.g. 4328).
    Integer,
    /// Render as a raw string (e.g. runid "my_runtag").
    Str,
    /// Render as a string wrapped in single quotes (e.g. relstring "'1-0-1'").
    QuotedStr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Str(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EvaluationType {
    Standard,
    JudgmentGroups,
    Preferences,
}


#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub query_flag: bool,
    pub summary_flag: bool,
    pub relevance_level: i64,
    pub average_complete_flag: bool,
    pub judged_docs_only_flag: bool,
    pub max_num_docs_per_topic: usize,
    pub num_docs_in_coll: usize,
    pub global_gains: Option<crate::metrics::common::GainsConfig>,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            query_flag: false,
            summary_flag: true,
            relevance_level: 1,
            average_complete_flag: false,
            judged_docs_only_flag: false,
            max_num_docs_per_topic: usize::MAX,
            num_docs_in_coll: 0,
            global_gains: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalState {
    Standard(QueryEvalState),
    JudgmentGroups(Vec<QueryEvalState>),
}

impl EvalState {
    /// Return the standard QueryEvalState, or the first JG state if in JudgmentGroups mode.
    pub fn get_standard(&self) -> Option<&QueryEvalState> {
        match self {
            EvalState::Standard(q) => Some(q),
            EvalState::JudgmentGroups(jgs) => jgs.first(),
        }
    }
}



pub trait Measure: Send + Sync {
    /// Unique root name of the measure (e.g. "map", "P").
    fn name(&self) -> &'static str;

    /// Short single-sentence summary of the measure (e.g. "Mean Average Precision").
    fn short_description(&self) -> &'static str;

    /// Detailed description/explanation of the measure.
    fn explanation(&self) -> &'static str;

    /// Printing format for the metric's values.
    fn format(&self) -> ValueFormat;

    /// The category of relevance judgments required by this measure.
    fn eval_type(&self) -> EvaluationType;

    /// Returns the exact sub-metric names to be calculated (e.g., `["P_5", "P_10", "P_15"]`).
    fn sub_metrics(&self) -> Vec<String>;

    /// Whether this measure should be reported in the summary averages at the end.
    fn is_summary_enabled(&self) -> bool {
        true
    }

    /// Whether this measure should be reported for individual queries.
    fn is_query_enabled(&self) -> bool {
        true
    }

    /// The invariants this measure claims to uphold (self-documentation,
    /// verified by the uniform invariant test). Defaults to the standard
    /// unit-interval bundle for normalized ranking measures; measures whose
    /// behavior differs (counts, unnormalized scores, signed, string) override.
    fn invariants(&self) -> &'static [Invariant] {
        invariants::STANDARD_UNIT_INTERVAL
    }

    /// Returns the initial values for the running totals of this measure.
    fn initial_values(&self) -> Vec<MetricValue>;

    /// Calculate the score(s) for a single query.
    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue>;

    /// Accumulate a single query's scores into a running total.
    fn accumulate(&self, q_scores: &[MetricValue], running_totals: &mut [MetricValue]) {
        for (i, score) in q_scores.iter().enumerate() {
            match (score, &mut running_totals[i]) {
                (MetricValue::Float(s), MetricValue::Float(t)) => *t += s,
                (MetricValue::Integer(s), MetricValue::Integer(t)) => *t += s,
                _ => {} // Strings and non-matching types do not accumulate
            }
        }
    }

    /// Calculate the final summary score from the accumulated totals.
    fn average(&self, config: &EvalConfig, running_totals: &mut [MetricValue], num_queries_evaluated: usize, total_qrels_queries: usize) {
        let denominator = if config.average_complete_flag {
            total_qrels_queries
        } else {
            num_queries_evaluated
        };
        if denominator > 0 {
            for total in running_totals.iter_mut() {
                if let MetricValue::Float(t) = total {
                    *t /= denominator as f64;
                }
            }
        }
    }
}


