use crate::metrics::EvalConfig;
use std::collections::HashMap;
use std::fmt;

/// Distinguishes CLI/specification syntax errors (exit 1) from invalid
/// parameter values for a known measure (exit 2), matching C `trec_eval`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeasureErrorKind {
    Spec,
    Param,
}

/// Error returned when a measure specification or parameter string is invalid.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureParseError {
    pub kind: MeasureErrorKind,
    pub message: String,
}

impl MeasureParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: MeasureErrorKind::Param,
            message: message.into(),
        }
    }

    pub fn spec(message: impl Into<String>) -> Self {
        Self {
            kind: MeasureErrorKind::Spec,
            message: message.into(),
        }
    }
}

impl fmt::Display for MeasureParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MeasureParseError {}

/// Parse a comma-separated list of integer cutoffs (e.g. "5,10,20").
/// `None` yields the provided defaults; `Some("")` is rejected.
pub fn parse_int_cutoffs(params: Option<&str>, default: &[usize]) -> Result<Vec<usize>, MeasureParseError> {
    let params = match params {
        None => return Ok(default.to_vec()),
        Some(s) => s,
    };
    let mut list = Vec::new();
    for s in params.split(',') {
        let tok = s.trim();
        match tok.parse::<usize>() {
            Ok(v) => list.push(v),
            Err(_) => return Err(MeasureParseError::new(format!("invalid integer cutoff '{}'", tok))),
        }
    }
    Ok(list)
}

/// Parse a comma-separated list of float values (e.g. "0.0,0.5,1.0").
/// `None` yields the provided defaults; `Some("")` is rejected.
pub fn parse_float_cutoffs(params: Option<&str>, default: &[f64]) -> Result<Vec<f64>, MeasureParseError> {
    let params = match params {
        None => return Ok(default.to_vec()),
        Some(s) => s,
    };
    let mut list = Vec::new();
    for s in params.split(',') {
        let tok = s.trim();
        match tok.parse::<f64>() {
            Ok(v) => list.push(v),
            Err(_) => return Err(MeasureParseError::new(format!("invalid float value '{}'", tok))),
        }
    }
    Ok(list)
}

/// Parse a comma-separated list of `key=value` pairs (e.g. "p=0.9").
/// Values are returned as raw strings for the caller to interpret. An empty
/// string yields an empty list.
pub fn parse_key_values(params: &str) -> Result<Vec<(String, String)>, MeasureParseError> {
    if params.is_empty() {
        return Ok(Vec::new());
    }
    let mut pairs = Vec::new();
    for part in params.split(',') {
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| MeasureParseError::new(format!("expected key=value, got '{}'", part.trim())))?;
        pairs.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(pairs)
}

/// Sort and validate positive integer rank cutoffs.
/// Rejects empty lists, zero cutoffs, cutoffs above `i64::MAX`, and duplicates.
pub(crate) fn validate_rank_cutoffs(mut cutoffs: Vec<usize>) -> Result<Vec<usize>, MeasureParseError> {
    if cutoffs.is_empty() {
        return Err(MeasureParseError::new("cutoff list must not be empty"));
    }
    for &c in &cutoffs {
        if c == 0 {
            return Err(MeasureParseError::new("cutoff must be a positive integer (got 0)"));
        }
        if c > i64::MAX as usize {
            return Err(MeasureParseError::new(format!(
                "illegal cutoff '{}' (exceeds maximum)",
                c
            )));
        }
    }
    cutoffs.sort_unstable();
    for w in cutoffs.windows(2) {
        if w[0] == w[1] {
            return Err(MeasureParseError::new(format!("duplicate cutoff '{}'", w[0])));
        }
    }
    Ok(cutoffs)
}

/// Sort and validate finite float cutoffs, rejecting empty lists, non-finite
/// values, out-of-range values, and exact duplicates (including `-0.0` vs `0.0`).
pub(crate) fn validate_float_cutoffs(
    mut cutoffs: Vec<f64>,
    label: &str,
    in_range: impl Fn(f64) -> bool,
    range_desc: &str,
) -> Result<Vec<f64>, MeasureParseError> {
    if cutoffs.is_empty() {
        return Err(MeasureParseError::new(format!("{} list must not be empty", label)));
    }
    for &v in &cutoffs {
        if !v.is_finite() {
            return Err(MeasureParseError::new(format!("non-finite {} '{}'", label, v)));
        }
        if !in_range(v) {
            return Err(MeasureParseError::new(format!(
                "{} '{}' must be {}",
                label, v, range_desc
            )));
        }
    }
    cutoffs.sort_by(f64::total_cmp);
    for w in cutoffs.windows(2) {
        if w[0] == w[1] {
            return Err(MeasureParseError::new(format!("duplicate {} '{}'", label, w[0])));
        }
    }
    Ok(cutoffs)
}

/// Helper function to count the number of relevant documents retrieved up to a certain rank limit.
pub(crate) fn count_relevant_retrieved_up_to(results_rel_list: &[i64], limit: usize, config: &EvalConfig) -> usize {
    let actual_limit = std::cmp::min(limit, results_rel_list.len());
    results_rel_list[..actual_limit]
        .iter()
        .filter(|&&rel| rel >= config.relevance_level)
        .count()
}

#[derive(Debug, Clone)]
pub struct GainsConfig {
    pub custom_gains: HashMap<i64, f64>,
}

impl GainsConfig {
    pub fn parse(params_str: &str) -> Self {
        let mut custom_gains = HashMap::new();
        if !params_str.is_empty() {
            for pair_str in params_str.split(',') {
                let parts: Vec<&str> = pair_str.split('=').collect();
                if parts.len() == 2 {
                    if let (Ok(rel_level), Ok(gain)) = (parts[0].trim().parse::<i64>(), parts[1].trim().parse::<f64>()) {
                        custom_gains.insert(rel_level, gain);
                    }
                }
            }
        }
        Self { custom_gains }
    }
}

#[derive(Debug, Clone)]
pub struct RelGain {
    pub rel_level: i64,
    pub gain: f64,
    pub num_at_level: usize,
}

#[derive(Debug, Clone)]
pub struct Gains {
    pub rel_gains: Vec<RelGain>,
    pub _total_num_at_levels: usize,
}

impl Gains {
    pub fn setup(config: &GainsConfig, rel_levels: &[usize]) -> Self {
        let mut rel_gains = Vec::new();

        // 1. First add the custom gains specified in parameters
        for (&rel_level, &gain) in &config.custom_gains {
            let num_at_level = if rel_level >= 0 && (rel_level as usize) < rel_levels.len() {
                rel_levels[rel_level as usize]
            } else {
                0
            };
            rel_gains.push(RelGain {
                rel_level,
                gain,
                num_at_level,
            });
        }

        // 2. Then add all other relevance levels that were NOT specified in custom_gains
        for i in 0..rel_levels.len() {
            let rel_level = i as i64;
            if !config.custom_gains.contains_key(&rel_level) {
                rel_gains.push(RelGain {
                    rel_level,
                    gain: rel_level as f64,
                    num_at_level: rel_levels[i],
                });
            }
        }

        // 3. Sort by increasing gain value using total_cmp for determinism
        rel_gains.sort_by(|a, b| a.gain.total_cmp(&b.gain));

        let _total_num_at_levels = rel_gains.iter().map(|g| g.num_at_level).sum();

        Gains {
            rel_gains,
            _total_num_at_levels,
        }
    }

    pub fn get_gain(&self, rel_level: i64) -> f64 {
        for g in &self.rel_gains {
            if g.rel_level == rel_level {
                return g.gain;
            }
        }
        0.0
    }
}

#[cfg(test)]
pub(crate) fn make_mock_state(results_rel_list: Vec<i64>, num_rel: i64) -> crate::eval::alignment::QueryEvalState {
    let num_ret = results_rel_list.len();
    let num_rel_ret = results_rel_list.iter().filter(|&&r| r >= 1).count(); // assume relevance level cutoff >= 1
    
    let mut rel_levels = vec![0; 2];
    rel_levels[1] = num_rel as usize;
    rel_levels[0] = 100; // default count of non-relevant docs
    
    crate::eval::alignment::QueryEvalState {
        qid: "mock_topic".to_string(),
        run_id: "mock_run".to_string(),
        results_rel_list,
        num_ret,
        num_rel: num_rel as usize,
        num_rel_ret,
        num_nonpool: 0,
        num_unjudged_in_pool: 0,
        rel_levels,
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn int_cutoffs_none_uses_default() {
        assert_eq!(parse_int_cutoffs(None, &[5, 10, 20]).unwrap(), vec![5, 10, 20]);
    }

    #[test]
    fn int_cutoffs_empty_some_is_rejected() {
        assert!(parse_int_cutoffs(Some(""), &[5, 10, 20]).is_err());
    }

    #[test]
    fn int_cutoffs_parses_and_trims() {
        assert_eq!(parse_int_cutoffs(Some("5, 10 ,20"), &[]).unwrap(), vec![5, 10, 20]);
    }

    #[test]
    fn int_cutoffs_rejects_bad_token() {
        assert!(parse_int_cutoffs(Some("5,x,20"), &[]).is_err());
        assert!(parse_int_cutoffs(Some("5,-1"), &[]).is_err());
    }

    #[test]
    fn float_cutoffs_none_uses_default() {
        assert_eq!(parse_float_cutoffs(None, &[0.0, 1.0]).unwrap(), vec![0.0, 1.0]);
    }

    #[test]
    fn float_cutoffs_empty_some_is_rejected() {
        assert!(parse_float_cutoffs(Some(""), &[0.0, 1.0]).is_err());
    }

    #[test]
    fn float_cutoffs_parses_and_trims() {
        assert_eq!(parse_float_cutoffs(Some("0.0, 0.5 ,1.0"), &[]).unwrap(), vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn float_cutoffs_rejects_bad_token() {
        assert!(parse_float_cutoffs(Some("0.0,nope"), &[]).is_err());
    }

    #[test]
    fn key_values_empty_is_empty() {
        assert_eq!(parse_key_values("").unwrap(), Vec::<(String, String)>::new());
    }

    #[test]
    fn key_values_parses_and_trims() {
        assert_eq!(
            parse_key_values("p=0.9, q = 2").unwrap(),
            vec![("p".to_string(), "0.9".to_string()), ("q".to_string(), "2".to_string())]
        );
    }

    #[test]
    fn key_values_rejects_missing_equals() {
        assert!(parse_key_values("p0.9").is_err());
    }
}
