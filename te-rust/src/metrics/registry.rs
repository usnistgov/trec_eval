//! Central registry of measures, the single source of truth for measure names,
//! parameter parsing, and construction. Analogous to C `trec_eval`'s
//! `te_trec_measures[]` table in `measures.c`.
//!
//! Each measure is described by a [`MeasureSpec`] carrying its name, status, and
//! a stateless factory function that builds an instance from the parameter
//! string that follows the `.` in a measure argument (e.g. the `10,20` in
//! `P.10,20`). Groups such as `official` and `all_trec` are name lists resolved
//! against this table.

use crate::metrics::common::{
    parse_float_cutoffs, parse_int_cutoffs, parse_key_values, MeasureParseError,
};
use crate::metrics::{self, Measure};

/// Development/support status of a measure (ISSUES #24).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeasureStatus {
    /// Standard, supported measure.
    Production,
    /// Experimental measure; use with care.
    Experimental,
    /// Obsolete measure kept for compatibility.
    Obsolete,
}

impl MeasureStatus {
    /// Short label for help output; empty for production measures.
    pub fn label(&self) -> &'static str {
        match self {
            MeasureStatus::Production => "",
            MeasureStatus::Experimental => "experimental",
            MeasureStatus::Obsolete => "obsolete",
        }
    }
}

/// Factory building a measure instance from its parameter string.
type MeasureFactory = fn(&str) -> Result<Box<dyn Measure>, MeasureParseError>;

/// Static description of a single measure root.
pub struct MeasureSpec {
    pub name: &'static str,
    pub status: MeasureStatus,
    /// Parameter usage line shown in help, e.g. describing cutoffs or gains and
    /// their defaults. Empty for measures that take no parameters.
    pub usage: &'static str,
    pub factory: MeasureFactory,
}

// Default cutoff/parameter sets, matching the previous main.rs behavior exactly.
const DEFAULT_RANK_CUTOFFS: &[usize] = &[5, 10, 15, 20, 30, 100, 200, 500, 1000];
const DEFAULT_SUCCESS_CUTOFFS: &[usize] = &[1, 5, 10];
const DEFAULT_UNJ_CUTOFFS: &[usize] = &[5, 10, 20];
const DEFAULT_RECALL_LEVELS: &[f64] = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
const DEFAULT_RPREC_MULT: &[f64] = &[0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0];
const DEFAULT_UTILITY_COEFFS: &[f64] = &[1.0, -1.0, 0.0, 0.0];

/// Extract the RBP-style `p=<float>` parameter, defaulting to 0.9.
fn parse_rbp_p(params: &str) -> Result<f64, MeasureParseError> {
    let mut p = 0.9;
    for (k, v) in parse_key_values(params)? {
        if k == "p" {
            p = v
                .parse::<f64>()
                .map_err(|_| MeasureParseError::new(format!("invalid float parameter '{}'", v)))?;
        }
    }
    Ok(p)
}

/// The full measure registry table.
pub fn registry() -> &'static [MeasureSpec] {
    use MeasureStatus::*;
    &[
        MeasureSpec { name: "runid", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::runid::RunIdMeasure::new())) },
        MeasureSpec { name: "num_q", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::num_q::NumQMeasure::new())) },
        MeasureSpec { name: "num_ret", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::num_ret::NumRetMeasure::new())) },
        MeasureSpec { name: "num_rel", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::num_rel::NumRelMeasure::new())) },
        MeasureSpec { name: "num_rel_ret", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::num_rel_ret::NumRelRetMeasure::new())) },
        MeasureSpec { name: "num_nonrel_judged_ret", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::num_nonrel_judged_ret::NumNonrelJudgedRetMeasure::new())) },
        MeasureSpec { name: "map", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::map::MapMeasure::new())) },
        MeasureSpec { name: "gm_map", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::gm_map::GMMapMeasure::new())) },
        MeasureSpec { name: "Rprec", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::rprec::RprecMeasure::new())) },
        MeasureSpec { name: "recip_rank", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::recip_rank::RecipRankMeasure::new())) },
        MeasureSpec { name: "bpref", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::bpref::BprefMeasure::new())) },
        MeasureSpec { name: "gm_bpref", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::gm_bpref::GMBprefMeasure::new())) },
        MeasureSpec { name: "infAP", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::infap::InfAPMeasure::new())) },
        MeasureSpec {
            name: "P",
            status: Production,
            usage: "P[.<c1,c2,...>]  precision at integer rank cutoffs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::precision::PrecisionCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "recall",
            status: Production,
            usage: "recall[.<c1,c2,...>]  recall at integer rank cutoffs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::recall::RecallCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "ndcg_cut",
            status: Production,
            usage: "ndcg_cut[.<c1,c2,...>]  nDCG at integer rank cutoffs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::ndcg_cut::NdcgCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "map_cut",
            status: Production,
            usage: "map_cut[.<c1,c2,...>]  MAP at integer rank cutoffs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::map_cut::MapCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "relative_P",
            status: Production,
            usage: "relative_P[.<c1,c2,...>]  relative precision at integer rank cutoffs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::relative_p::RelativePMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "success",
            status: Production,
            usage: "success[.<c1,c2,...>]  success at integer rank cutoffs (default: 1,5,10)",
            factory: |p| Ok(Box::new(metrics::success::SuccessCutMeasure::new(parse_int_cutoffs(p, DEFAULT_SUCCESS_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "unj",
            status: Production,
            usage: "unj[.<c1,c2,...>]  unjudged fraction at integer rank cutoffs (default: 5,10,20)",
            factory: |p| Ok(Box::new(metrics::unj::UnjMeasure::new(parse_int_cutoffs(p, DEFAULT_UNJ_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "11pt_avg",
            status: Production,
            usage: "11pt_avg[.<r1,r2,...>]  interpolated precision at recall levels (default: 0.0,0.1,...,1.0)",
            factory: |p| Ok(Box::new(metrics::avg_11pt::Avg11PtMeasure::new(parse_float_cutoffs(p, DEFAULT_RECALL_LEVELS)?, p))),
        },
        MeasureSpec {
            name: "iprec_at_recall",
            status: Production,
            usage: "iprec_at_recall[.<r1,r2,...>]  interpolated precision at recall levels (default: 0.0,0.1,...,1.0)",
            factory: |p| Ok(Box::new(metrics::iprec_at_recall::IprecAtRecallMeasure::new(parse_float_cutoffs(p, DEFAULT_RECALL_LEVELS)?))),
        },
        MeasureSpec {
            name: "Rprec_mult",
            status: Production,
            usage: "Rprec_mult[.<m1,m2,...>]  R-precision at multiples of R (default: 0.2,0.4,...,2.0)",
            factory: |p| Ok(Box::new(metrics::rprec_mult::RprecMultMeasure::new(parse_float_cutoffs(p, DEFAULT_RPREC_MULT)?))),
        },
        MeasureSpec {
            name: "utility",
            status: Production,
            usage: "utility.<rr,rn,nr,nn>  four utility coefficients (default: 1.0,-1.0,0.0,0.0)",
            factory: |p| {
                let coeffs = parse_float_cutoffs(p, DEFAULT_UTILITY_COEFFS)?;
                if coeffs.len() != 4 {
                    return Err(MeasureParseError::new("utility requires exactly 4 coefficients"));
                }
                Ok(Box::new(metrics::utility::UtilityMeasure::new(coeffs, p)))
            },
        },
        MeasureSpec {
            name: "set_F",
            status: Production,
            usage: "set_F[.<beta>]  F-measure beta weighting (default: 1.0)",
            factory: |p| {
                let beta = if p.is_empty() {
                    1.0
                } else {
                    p.parse::<f64>().map_err(|_| MeasureParseError::new(format!("invalid float parameter '{}'", p)))?
                };
                Ok(Box::new(metrics::set_f::SetFMeasure::new(beta, p)))
            },
        },
        MeasureSpec {
            name: "relstring",
            status: Production,
            usage: "relstring[.<len>]  relevance string length (default: 10)",
            factory: |p| {
                let len = if p.is_empty() {
                    10
                } else {
                    p.trim().parse::<usize>().map_err(|_| MeasureParseError::new(format!("invalid length '{}'", p)))?
                };
                Ok(Box::new(metrics::relstring::RelstringMeasure::new(len, p)))
            },
        },
        MeasureSpec { name: "set_P", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::set_p::SetPMeasure::new())) },
        MeasureSpec { name: "set_recall", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::set_recall::SetRecallMeasure::new())) },
        MeasureSpec { name: "set_relative_P", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::set_relative_p::SetRelativePMeasure::new())) },
        MeasureSpec { name: "set_map", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::set_map::SetMapMeasure::new())) },
        MeasureSpec { name: "G", status: Production, usage: "G[.<rel>=<gain>,...]  optional relevance-to-gain mapping (default: gain = relevance level)", factory: |p| Ok(Box::new(metrics::g::GMeasure::new(p))) },
        MeasureSpec { name: "ndcg", status: Production, usage: "ndcg[.<rel>=<gain>,...]  optional relevance-to-gain mapping (default: gain = relevance level)", factory: |p| Ok(Box::new(metrics::ndcg::NdcgMeasure::new(p))) },
        MeasureSpec { name: "ndcg_rel", status: Production, usage: "ndcg_rel[.<rel>=<gain>,...]  optional relevance-to-gain mapping (default: gain = relevance level)", factory: |p| Ok(Box::new(metrics::ndcg_rel::NdcgRelMeasure::new(p))) },
        MeasureSpec { name: "Rndcg", status: Production, usage: "Rndcg[.<rel>=<gain>,...]  optional relevance-to-gain mapping (default: gain = relevance level)", factory: |p| Ok(Box::new(metrics::rndcg::RndcgMeasure::new(p))) },
        MeasureSpec { name: "ndcg_p", status: Production, usage: "ndcg_p[.<rel>=<gain>,...]  optional relevance-to-gain mapping (default: gain = relevance level)", factory: |p| Ok(Box::new(metrics::ndcg_p::NdcgPMeasure::new(p))) },
        MeasureSpec { name: "rbp", status: Production, usage: "rbp[.p=<float>]  persistence parameter p (default: 0.9)", factory: |p| Ok(Box::new(metrics::rbp::RbpMeasure::new(parse_rbp_p(p)?, p))) },
        MeasureSpec { name: "rbp_resid", status: Production, usage: "rbp_resid[.p=<float>]  persistence parameter p (default: 0.9)", factory: |p| Ok(Box::new(metrics::rbp_resid::RbpResidMeasure::new(parse_rbp_p(p)?, p))) },
        MeasureSpec { name: "map_avgjg", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::map_avgjg::MapAvgjgMeasure::new())) },
        MeasureSpec {
            name: "P_avgjg",
            status: Production,
            usage: "P_avgjg[.<c1,c2,...>]  precision at integer rank cutoffs averaged over JGs (default: 5,10,15,20,30,100,200,500,1000)",
            factory: |p| Ok(Box::new(metrics::precision_avgjg::PrecisionAvgjgMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "Rprec_mult_avgjg",
            status: Production,
            usage: "Rprec_mult_avgjg[.<m1,m2,...>]  R-precision at multiples of R averaged over JGs (default: 0.2,0.4,...,2.0)",
            factory: |p| Ok(Box::new(metrics::rprec_mult_avgjg::RprecMultAvgjgMeasure::new(parse_float_cutoffs(p, DEFAULT_RPREC_MULT)?))),
        },
        MeasureSpec { name: "yaap", status: Experimental, usage: "", factory: |_| Ok(Box::new(metrics::yaap::YaapMeasure::new())) },

        MeasureSpec { name: "binG", status: Production, usage: "", factory: |_| Ok(Box::new(metrics::bin_g::BinGMeasure::new())) },
    ]
}

/// Look up a measure spec by its root name (case-insensitive).
pub fn find_spec(name: &str) -> Option<&'static MeasureSpec> {
    let lookup = match name.to_ascii_lowercase().as_str() {
        "mrr" | "rr" => "recip_rank",
        "precision" => "P",
        _ => name,
    };
    registry().iter().find(|s| s.name.eq_ignore_ascii_case(lookup))
}

/// Expand a predefined group name into its list of member measure names.
/// Returns `None` if the name is not a known group.
pub fn expand_group(name: &str) -> Option<&'static [&'static str]> {
    match name.to_ascii_lowercase().as_str() {
        "official" => Some(&[
            "runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map",
            "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P",
        ]),
        "set" => Some(&[
            "runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "utility", "set_P",
            "set_relative_P", "set_recall", "set_map", "set_F",
        ]),
        "qrels_jg" => Some(&["runid", "num_q", "map_avgjg", "P_avgjg", "Rprec_mult_avgjg"]),
        "all_trec" => Some(&[
            "runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map",
            "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P", "relstring",
            "recall", "infAP", "gm_bpref", "Rprec_mult", "utility", "11pt_avg",
            "binG", "G", "ndcg", "ndcg_rel", "Rndcg", "ndcg_cut", "map_cut",
            "relative_P", "success", "set_P", "set_relative_P", "set_recall",
            "set_map", "set_F", "num_nonrel_judged_ret", "rbp", "rbp_resid", "unj",
        ]),
        _ => None,
    }
}

/// Canonicalize an input measure string (e.g. `ndcg@10`, `P.5,10`, `P_5`, `MRR`) into `(root, params)`.
fn canonicalize_measure_spec(arg: &str) -> (String, String) {
    if let Some((root, cutoffs)) = arg.split_once('@') {
        let root_lower = root.to_ascii_lowercase();
        match root_lower.as_str() {
            "ndcg" => ("ndcg_cut".to_string(), cutoffs.to_string()),
            "map" => ("map_cut".to_string(), cutoffs.to_string()),
            "p" | "precision" => ("P".to_string(), cutoffs.to_string()),
            "recall" => ("recall".to_string(), cutoffs.to_string()),
            "success" => ("success".to_string(), cutoffs.to_string()),
            "unj" => ("unj".to_string(), cutoffs.to_string()),
            "relative_p" => ("relative_P".to_string(), cutoffs.to_string()),
            _ => (root.to_string(), cutoffs.to_string()),
        }
    } else if let Some((root, params)) = arg.split_once('.') {
        (root.to_string(), params.to_string())
    } else {
        // Check for underscore cutoffs on known prefix names (e.g. P_5, ndcg_cut_10, map_cut_100, recall_10, success_5, unj_10)
        let arg_lower = arg.to_ascii_lowercase();
        if let Some(rest) = arg_lower.strip_prefix("p_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("P".to_string(), rest.to_string());
            }
        }
        if let Some(rest) = arg_lower.strip_prefix("p_avgjg_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("P_avgjg".to_string(), rest.to_string());
            }
        }

        if let Some(rest) = arg_lower.strip_prefix("ndcg_cut_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("ndcg_cut".to_string(), rest.to_string());
            }
        }
        if let Some(rest) = arg_lower.strip_prefix("map_cut_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("map_cut".to_string(), rest.to_string());
            }
        }
        if let Some(rest) = arg_lower.strip_prefix("recall_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("recall".to_string(), rest.to_string());
            }
        }
        if let Some(rest) = arg_lower.strip_prefix("success_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("success".to_string(), rest.to_string());
            }
        }
        if let Some(rest) = arg_lower.strip_prefix("unj_") {
            if rest.chars().all(|c| c.is_ascii_digit() || c == ',') {
                return ("unj".to_string(), rest.to_string());
            }
        }
        (arg.to_string(), String::new())
    }
}


/// Resolve a list of requested measure arguments into constructed measures.
///
/// Each request is either a group name (expanded via [`expand_group`]) or a
/// `root[.params]` (or alias like `root@cutoff`) measure specification.
/// Parameters are passed to the measure's factory. Errors carry the offending argument.
pub fn resolve_measures(requested: &[String]) -> Result<Vec<Box<dyn Measure>>, MeasureParseError> {
    // 1. Expand any groups into concrete measure argument strings.
    let mut expanded: Vec<String> = Vec::new();
    for req in requested {
        match expand_group(req) {
            Some(members) => expanded.extend(members.iter().map(|m| m.to_string())),
            None => expanded.push(req.clone()),
        }
    }

    // 2. Build each measure from its root and parameter string.
    let mut measures: Vec<Box<dyn Measure>> = Vec::with_capacity(expanded.len());
    for arg in &expanded {
        let (root, params) = canonicalize_measure_spec(arg);
        let spec = find_spec(&root)
            .ok_or_else(|| MeasureParseError::new(format!("unknown measure '{}'", arg)))?;
        let measure = (spec.factory)(&params)
            .map_err(|e| MeasureParseError::new(format!("measure '{}': {}", arg, e)))?;
        measures.push(measure);
    }
    Ok(measures)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_factory_builds_with_defaults() {
        for spec in registry() {
            let m = (spec.factory)("").unwrap_or_else(|e| panic!("{} failed: {}", spec.name, e));
            assert_eq!(m.name(), spec.name, "spec name must match measure name");
        }
    }

    #[test]
    fn usage_lines_start_with_measure_name() {
        for spec in registry() {
            if !spec.usage.is_empty() {
                assert!(
                    spec.usage.starts_with(spec.name),
                    "usage for '{}' should start with its name: {}",
                    spec.name,
                    spec.usage
                );
            }
        }
    }

    #[test]
    fn groups_contain_only_production_measures() {
        for group in ["official", "set", "qrels_jg", "all_trec"] {
            for name in expand_group(group).unwrap() {

                let spec = find_spec(name).unwrap_or_else(|| panic!("group '{}' names unknown measure '{}'", group, name));
                assert_eq!(
                    spec.status,
                    MeasureStatus::Production,
                    "group '{}' must not contain non-production measure '{}'",
                    group,
                    name
                );
            }
        }
    }

    #[test]
    fn alias_and_cutoff_at_syntax_resolves() {
        let measures = resolve_measures(&[
            "ndcg@10".to_string(),
            "MAP@100".to_string(),
            "P@5,10".to_string(),
            "mrr".to_string(),
            "recip_rank".to_string(),
        ]).unwrap();
        assert_eq!(measures.len(), 5);
        assert_eq!(measures[0].name(), "ndcg_cut");
        assert_eq!(measures[0].sub_metrics(), vec!["ndcg_cut_10"]);
        assert_eq!(measures[1].name(), "map_cut");
        assert_eq!(measures[1].sub_metrics(), vec!["map_cut_100"]);
        assert_eq!(measures[2].name(), "P");
        assert_eq!(measures[2].sub_metrics(), vec!["P_5", "P_10"]);
        assert_eq!(measures[3].name(), "recip_rank");
        assert_eq!(measures[4].name(), "recip_rank");
    }


    #[test]
    fn official_group_expands() {
        let ms = resolve_measures(&["official".to_string()]).unwrap();
        let names: Vec<_> = ms.iter().map(|m| m.name()).collect();
        assert_eq!(
            names,
            vec![
                "runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map",
                "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P"
            ]
        );
    }

    #[test]
    fn parameterized_measure_parses() {
        let ms = resolve_measures(&["P.10,20".to_string()]).unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].sub_metrics(), vec!["P_10".to_string(), "P_20".to_string()]);
    }

    #[test]
    fn unknown_measure_errors() {
        assert!(resolve_measures(&["nope".to_string()]).is_err());
    }

    #[test]
    fn bad_param_errors_with_context() {
        match resolve_measures(&["P.x".to_string()]) {
            Err(e) => assert!(e.message.contains("P.x"), "error should name the argument: {}", e.message),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn utility_requires_four_coeffs() {
        assert!(resolve_measures(&["utility.1.0,2.0".to_string()]).is_err());
    }
}
