//! Declarative measure invariants (ISSUES #13).
//!
//! An [`Invariant`] is a property a measure claims to satisfy, declared via
//! [`Measure::invariants`](crate::metrics::Measure::invariants). This is
//! self-documentation for the measure and is consumed only by the test suite
//! (see `checking` below) — it is never evaluated during real metric
//! computation. The uniform invariant test iterates the registry and verifies
//! every measure honors each invariant it declares, so shared boundary
//! properties are asserted once instead of being duplicated per measure.

/// A property a measure declares it upholds. Checked at test time only.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Invariant {
    /// All numeric sub-metric values are finite (no NaN or infinity), across
    /// standard and boundary inputs.
    Finite,
    /// No numeric sub-metric value is negative.
    NonNegative,
    /// On an empty ranking (no documents retrieved), every sub-metric is zero.
    EmptyRankingIsZero,
    /// On a query topic with zero relevant documents, every sub-metric is zero.
    ZeroRelevanceIsZero,
    /// All float sub-metric values lie within the closed unit interval [0, 1].
    UnitInterval,
}

/// The invariants satisfied by a typical ranking-quality measure (map, ndcg,
/// P, recall, ...). Most measures return this bundle unchanged.
pub const STANDARD: &[Invariant] = &[
    Invariant::Finite,
    Invariant::NonNegative,
    Invariant::EmptyRankingIsZero,
    Invariant::ZeroRelevanceIsZero,
];

/// Standard bundle plus the unit-interval bound [0, 1], for normalized measures.
pub const STANDARD_UNIT_INTERVAL: &[Invariant] = &[
    Invariant::Finite,
    Invariant::NonNegative,
    Invariant::EmptyRankingIsZero,
    Invariant::ZeroRelevanceIsZero,
    Invariant::UnitInterval,
];

/// Invariants for count measures (num_rel, num_nonrel_judged_ret, etc.) whose
/// values depend on the collection/pool rather than being zero on empty rankings.
pub const COUNTS: &[Invariant] = &[
    Invariant::Finite,
    Invariant::NonNegative,
];

/// Invariants for signed measures (utility) which may legitimately produce
/// negative values.
pub const SIGNED: &[Invariant] = &[
    Invariant::Finite,
    Invariant::EmptyRankingIsZero,
];

/// Invariants for transformed/log measures (gm_map, gm_bpref) whose per-query
/// scores are log-values and exponentiated in average().
pub const FINITE_ONLY: &[Invariant] = &[
    Invariant::Finite,
];

/// No invariants for string/tag measures (runid, relstring).
pub const NONE: &[Invariant] = &[];

#[cfg(test)]
pub(crate) mod checking {
    use super::Invariant;
    use crate::eval::alignment::QueryEvalState;
    use crate::metrics::{EvalConfig, EvalState, Measure, MetricValue};

    /// Build a `QueryEvalState` from a retrieved relevance list and total
    /// relevant count, sizing rel_levels to hold grades 0 and 1.
    fn state(results_rel_list: Vec<i64>, num_rel: usize) -> QueryEvalState {
        let num_ret = results_rel_list.len();
        let num_rel_ret = results_rel_list.iter().filter(|&&r| r >= 1).count();
        let num_nonpool = results_rel_list.iter().filter(|&&r| r == -1).count();
        let num_unjudged_in_pool = results_rel_list.iter().filter(|&&r| r == -2).count();
        let mut rel_levels = vec![0usize; 2];
        rel_levels[0] = 10;
        rel_levels[1] = num_rel;
        QueryEvalState {
            qid: "invariant".to_string(),
            run_id: "invariant_run".to_string(),
            results_rel_list,
            num_ret,
            num_rel,
            num_rel_ret,
            num_nonpool,
            num_unjudged_in_pool,
            rel_levels,
        }
    }

    /// Inputs exercised for the always-checked invariants (Finite, NonNegative,
    /// UnitInterval). Covers normal and boundary rankings.
    fn general_inputs() -> Vec<(&'static str, QueryEvalState)> {
        vec![
            ("typical ranking", state(vec![1, 0, 1, 0, 0], 3)),
            ("empty ranking, has relevant", state(vec![], 5)),
            ("zero-relevance topic", state(vec![0, 0, 0], 0)),
            ("no relevant retrieved", state(vec![0, 0, 0], 5)),
            ("no judged retrieved", state(vec![-1, -1, -1], 5)),
            ("single relevant", state(vec![1], 1)),
        ]
    }

    /// Check one invariant for one measure, returning Err(message) on violation.
    pub fn check(inv: Invariant, measure: &dyn Measure) -> Result<(), String> {
        let config = EvalConfig::default();
        match inv {
            Invariant::Finite => {
                for (label, st) in general_inputs() {
                    for v in measure.calc(&config, &EvalState::Standard(st)) {
                        if let MetricValue::Float(f) = v {
                            if !f.is_finite() {
                                return Err(format!("non-finite value {} for input '{}'", f, label));
                            }
                        }
                    }
                }
            }
            Invariant::NonNegative => {
                for (label, st) in general_inputs() {
                    for v in measure.calc(&config, &EvalState::Standard(st)) {
                        match v {
                            MetricValue::Float(f) if f < 0.0 => {
                                return Err(format!("negative value {} for input '{}'", f, label));
                            }
                            MetricValue::Integer(i) if i < 0 => {
                                return Err(format!("negative count {} for input '{}'", i, label));
                            }
                            _ => {}
                        }
                    }
                }
            }
            Invariant::UnitInterval => {
                for (label, st) in general_inputs() {
                    for v in measure.calc(&config, &EvalState::Standard(st)) {
                        if let MetricValue::Float(f) = v {
                            if !(0.0..=1.0).contains(&f) {
                                return Err(format!("value {} outside [0,1] for input '{}'", f, label));
                            }
                        }
                    }
                }
            }
            Invariant::EmptyRankingIsZero => {
                // Empty ranking with relevant docs in the collection.
                let st = state(vec![], 5);
                for v in measure.calc(&config, &EvalState::Standard(st)) {
                    match v {
                        MetricValue::Float(f) if f != 0.0 => {
                            return Err(format!("empty ranking produced {} (expected 0.0)", f));
                        }
                        MetricValue::Integer(i) if i != 0 => {
                            return Err(format!("empty ranking produced {} (expected 0)", i));
                        }
                        _ => {}
                    }
                }
            }
            Invariant::ZeroRelevanceIsZero => {
                // Non-empty ranking on a topic with zero relevant docs in judgments (all retrieved non-relevant).
                let st = state(vec![0, 0, 0, 0, 0], 0);
                for v in measure.calc(&config, &EvalState::Standard(st)) {
                    match v {
                        MetricValue::Float(f) if f != 0.0 => {
                            return Err(format!("zero-relevance topic produced {} (expected 0.0)", f));
                        }
                        MetricValue::Integer(i) if i != 0 => {
                            return Err(format!("zero-relevance topic produced {} (expected 0)", i));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn all_measures_uphold_declared_invariants() {
        use crate::metrics::registry::registry;

        for spec in registry() {
            let measure = (spec.factory)("")
                .unwrap_or_else(|e| panic!("registry factory for '{}' failed: {}", spec.name, e));

            for &inv in measure.invariants() {
                if let Err(msg) = check(inv, &*measure) {
                    panic!("measure '{}' violates invariant {:?}: {}", spec.name, inv, msg);
                }
            }
        }
    }
}
