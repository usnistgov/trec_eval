# `te-rust` Measure Parameter Parsing, Sorting, and Validation Design Document

This document specifies how `te-rust` parses, sorts, and validates measure parameters across all parameterized measures (addressing issues #57 and #162).

---

## 1. Guiding Principle: Syntax vs. Semantics

Parameter handling is split cleanly into two layers:

1. **Syntactic parsing (`registry.rs` and `common.rs`):**
   - Splits a CLI measure specification into `(root_name, Option<params_str>)`.
   - Parses raw comma-separated token lists (`parse_int_list`, `parse_float_list`, `parse_key_values`) without making assumptions about ordering, uniqueness, or value ranges.
   - Rejects malformed syntax (unknown measure name, empty suffix after `.`, `_`, or `@`, non-numeric tokens, missing `=` in `key=value` pairs).

2. **Measure-owned semantics (each measure's `new` constructor):**
   - Each measure owns the sorting, uniqueness, arity, and range rules for its own parameters.
   - Sorting happens in **one place**—inside the measure constructor—so CLI (`resolve_measures`), Python (`Evaluator`), and direct Rust library callers (`MapCutMeasure::new(vec![5, 2])`) all receive identical behavior and validation.
   - Every parameterized `Measure::new` returns `Result<Self, MeasureParseError>` (with `Default` implementations calling `.expect(...)` on known-valid defaults).

---

## 2. Error Classification and CLI Exit Codes

To match C `trec_eval`'s exit status conventions:

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeasureErrorKind {
    /// Unknown measure name, malformed measure spec (e.g. `P.`, `P_`, `P@`),
    /// or invalid CLI option value (e.g. `--global-gains 1=x`).
    /// CLI exits with status 1.
    Spec,
    /// Invalid parameter value for a known measure (e.g. cutoff `0`, duplicate
    /// cutoff, non-finite or out-of-range float, malformed gain pair in `-m ndcg.1=x`).
    /// CLI exits with status 2.
    Param,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureParseError {
    pub kind: MeasureErrorKind,
    pub message: String,
}
```

- `MeasureParseError::new(msg)` constructs a `MeasureErrorKind::Param` error (exit 2).
- `MeasureParseError::spec(msg)` constructs a `MeasureErrorKind::Spec` error (exit 1).
- In `main.rs`:
  - `resolve_measures` errors exit with `1` for `Spec` and `2` for `Param`.
  - `--global-gains` parsing errors exit with `1` (invalid CLI option value).
- In `te-python`: all `MeasureParseError` values map to `PyValueError` as before.

---

## 3. Specification Canonicalization (`registry.rs`)

`canonicalize_measure_spec(arg: &str)` changes from `(String, String)` to `Result<(String, Option<String>), MeasureParseError>`:

- `arg.split_once('@')`:
  - If the suffix is empty (e.g. `P@`), return `Err(MeasureParseError::spec(...))`.
  - Otherwise return `Ok((canonical_root, Some(cutoffs)))`.
- `arg.split_once('.')`:
  - If the suffix is empty (e.g. `P.`), return `Err(MeasureParseError::spec(...))`.
  - Otherwise return `Ok((root, Some(params)))`.
  - Note: `iprec_at_recall.1.` splits into `("iprec_at_recall", Some("1."))`, which is non-empty and parses as `1.0`—remaining legal.
- Underscore cutoff shortcuts (`P_`, `P_avgjg_`, `ndcg_cut_`, `map_cut_`, `recall_`, `success_`, `unj_`):
  - Require `!rest.is_empty()` in addition to `rest.chars().all(|c| c.is_ascii_digit() || c == ',')`.
  - `P_` therefore falls through to `find_spec("P_") == None` and returns `MeasureErrorKind::Spec` (`unknown measure 'P_'`, exit 1).
- Plain measure names (e.g. `P`, `map`): return `Ok((arg, None))`.

`MeasureFactory` becomes `fn(Option<&str>) -> Result<Box<dyn Measure>, MeasureParseError>`:
- `None` tells the factory to use the measure's default parameters.
- `Some(params)` passes the explicit parameter string to the measure's parser/constructor.
- Parameterless measures (e.g. `map`, `bpref`, `Rprec`) reject `Some(_)` with `MeasureParseError::new("does not accept parameters")`.

---

## 4. Parameter Families and Validation Rules

### Family 1: Integer Rank Cutoffs (8 measures)
**Measures:** `P`, `recall`, `ndcg_cut`, `map_cut`, `relative_P`, `success`, `unj`, `P_avgjg`

- **Constructor:** `new(mut cutoffs: Vec<usize>) -> Result<Self, MeasureParseError>`
- **Rules:**
  1. Must be non-empty.
  2. Every cutoff `c` must satisfy `1 <= c <= i64::MAX as usize` (rejects `0` and values exceeding C `long` max `9223372036854775807`).
  3. Sort ascending with `cutoffs.sort_unstable()` before constructing `sub_metrics`.
  4. Reject duplicates (`cutoffs.windows(2).any(|w| w[0] == w[1])`).
- **Behavior:** Matches C `trec_eval`'s `get_long_cutoffs` (`meas_init.c`) and fixes wrong values / out-of-order rows for unsorted cutoffs (#57).

### Family 2: Float Cutoffs — Multiple Output Rows (3 measures)
**Measures:** `iprec_at_recall`, `Rprec_mult`, `Rprec_mult_avgjg`

- **Constructor:** `new(mut cutoff_percents: Vec<f64>) -> Result<Self, MeasureParseError>`
- **Rules:**
  1. Must be non-empty.
  2. Every value `v` must be finite (`v.is_finite()`), rejecting `NaN`, `inf`, `-inf`, and float overflows like `1e400`.
  3. Domain range:
     - `iprec_at_recall`: `0.0 <= v && v <= 1.0`.
     - `Rprec_mult`, `Rprec_mult_avgjg`: `v > 0.0`.
  4. Sort ascending with `cutoff_percents.sort_by(f64::total_cmp)` before constructing `sub_metrics`.
  5. Reject exact float duplicates (`cutoff_percents.windows(2).any(|w| w[0] == w[1])`, which also catches `-0.0` vs `0.0`).
     - Distinct floats that round to the same `.2f` label (e.g. `0.999` and `1.001`) are allowed, matching C `trec_eval`.
- **Behavior:** Fixes wrong values for unsorted cutoffs (#57) and rejects invalid ranges / non-finite / duplicate cutoffs (#162).

### Family 3: Recall-Level Average — Single Output Row (1 measure)
**Measure:** `11pt_avg`

- **Constructor:** `new(mut cutoffs: Vec<f64>, params_str: &str) -> Result<Self, MeasureParseError>`
- **Rules:**
  1. Must be non-empty (prevents division by zero / `NaN`).
  2. Every value `v` must be finite (`v.is_finite()`) and within `0.0 <= v && v <= 1.0`.
  3. Sort ascending with `cutoffs.sort_by(f64::total_cmp)` for the backward-pass calculation.
  4. Reject exact float duplicates (`cutoffs.windows(2).any(|w| w[0] == w[1])`).
  5. Sub-metric name remains `"11pt_avg"` when `params_str.is_empty()`, or `"11pt_avg_<params_str>"` preserving the caller's `params_str`.
- **Intentional differences from C `trec_eval`:**
  1. `te-rust` sorts the recall levels so `11pt_avg.1.0,0.5` produces the correct average (matching `11pt_avg.0.5,1.0`), whereas C `trec_eval` currently has defect #142.
  2. `te-rust` rejects duplicate recall levels in `11pt_avg` (avoiding silent double-weighting from typos), whereas C `trec_eval` accepts them via `get_double_params`.

### Family 4: Positional Float Weights (1 measure)
**Measure:** `utility`

- **Constructor:** `new(params: Vec<f64>, params_str: &str) -> Result<Self, MeasureParseError>`
- **Rules:**
  1. Must have exact length `4` (`rr, rn, nr, nn`).
  2. Every coefficient must be finite (`v.is_finite()`).
  3. **Must not** be sorted; negative and duplicate weights are valid (default is `[1.0, -1.0, 0.0, 0.0]`).

### Family 5: Single Scalar Parameters (4 measures)
- **`set_F`:**
  - `new(beta: f64, params_str: &str) -> Result<Self, MeasureParseError>`
  - `beta` must be finite and `>= 0.0` (`0.0` is legal and equals `set_P`).
- **`relstring`:**
  - `new(len: usize, params_str: &str) -> Result<Self, MeasureParseError>`
  - `len` must satisfy `len <= i64::MAX as usize`. (`0` is allowed and produces `''`, matching C `trec_eval`.)
- **`rbp_resid`:**
  - Accepts only `p=<float>` (default `0.9`).
  - Rejects any unknown key (including numeric gain keys, which `rbp_resid` does not use), duplicate `p`, non-finite `p`, or `p` outside `[0.0, 1.0)`.
  - `RbpResidMeasure::new(p: f64, params_str: &str) -> Result<Self, MeasureParseError>` validates `p.is_finite() && (0.0..1.0).contains(&p)`.
- **`rbp`:**
  - Accepts `p=<float>` (default `0.9`) **and** optional numeric `<rel_level>=<gain>` pairs, matching C `trec_eval`'s `m_rbp.c`.
  - Rejects duplicate `p`, unknown non-integer keys (e.g. `q=1`), malformed/non-finite gains, duplicate relevance levels, and `p` outside `[0.0, 1.0)`.
  - Per-measure gains take precedence over `--global-gains`, matching `ndcg`/`G`.
  - `RbpMeasure::new(p: f64, gains_config: GainsConfig, params_str: &str) -> Result<Self, MeasureParseError>` (with a helper for parsing combined `p` + gain pairs).

### Family 6: Gain-Map Measures (5 measures + `--global-gains`)
**Measures:** `G`, `ndcg`, `ndcg_rel`, `Rndcg`, `ndcg_p`

- **`GainsConfig::parse(params_str: &str) -> Result<Self, MeasureParseError>`:**
  - Empty string yields an empty `custom_gains` map.
  - Otherwise splits on `,` and requires every entry to be `<i64>=<f64>`:
    - Rejects missing `=` (e.g. `5,5`).
    - Rejects non-`i64` relevance level (e.g. `x=1`).
    - Rejects non-`f64` or non-finite gain (e.g. `1=x`, `1=nan`, `1=inf`).
    - Rejects duplicate relevance levels (e.g. `1=2,1=5`, `1=2,01=3`).
- Each gain-map measure's `new(params_str: &str)` returns `Result<Self, MeasureParseError>`.
- `--global-gains` in `main.rs` calls `GainsConfig::parse` and exits with status `1` on error.

---

## 5. Implementation Plan (Step-by-Step)

Changes will be implemented incrementally on branch `fix/issue-57-sort-cutoffs`, with one focused step / one measure at a time for manual review:

1. **Shared infrastructure:** `MeasureErrorKind` / `MeasureParseError`, `Option<&str>` in `registry.rs` (`canonicalize_measure_spec`, `MeasureFactory`), exit code mapping in `main.rs`, and raw parsers in `common.rs`.
2. **Family 3:** `avg_11pt.rs`
3. **Family 1 (one measure per step):** `map_cut.rs`, `ndcg_cut.rs`, `relative_p.rs`, `unj.rs`, `precision.rs`, `recall.rs`, `success.rs`, `precision_avgjg.rs`
4. **Family 2 (one measure per step):** `iprec_at_recall.rs`, `rprec_mult.rs`, `rprec_mult_avgjg.rs`
5. **Family 4:** `utility.rs`
6. **Family 5 (one measure per step):** `set_f.rs`, `relstring.rs`, `rbp_resid.rs`, `rbp.rs`
7. **Family 6 (one measure per step):** `GainsConfig::parse` + `main.rs`/`te-python` `--global-gains`, then `g.rs`, `ndcg.rs`, `ndcg_rel.rs`, `rndcg.rs`, `ndcg_p.rs`
8. **Final verification:** Full `cargo test` suite, regression tests, and CLI reproduction checks from issues #57 and #162.
