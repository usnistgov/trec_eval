# Intentional Differences from C `trec_eval`

`te-rust` is designed to produce numerically identical evaluation scores to reference C `trec_eval` (9.x/10.x) on valid inputs and standard command-line usage. Where `te-rust` deliberately diverges from C `trec_eval`, it does so either to reject malformed or out-of-domain parameters that C `trec_eval` silently accepted, to fix known defects in C `trec_eval`, or to add backward-compatible usability features.

This page catalogs those intentional differences across the CLI, Rust library, and Python API.

---

## 1. Stricter Measure Parameter Validation

C `trec_eval` parses many measure parameters with permissive C functions such as `atof`, silently skipping unparseable tokens, coercing non-numeric strings to `0.0`, or accepting out-of-range values that yield `nan` or nonsensical scores. `te-rust` validates all measure parameters upfront—both on the CLI and in direct Rust/Python constructors—and returns an error rather than printing questionable results.

On the CLI, syntax/specification errors (such as unknown measures, empty suffixes after `.`/`_`/`@`, or invalid `--global-gains`) exit with status **`1`**, while invalid parameter values for a known measure exit with status **`2`** with a message naming the offending argument. In Python (`trec-eval`), all parameter errors raise `ValueError`.

### `11pt_avg` Recall Levels
- **Sorting:** `te-rust` sorts the recall-level list ascending before computing interpolated precision, so `-m 11pt_avg.1.0,0.5` produces the same average as `-m 11pt_avg.0.5,1.0`. C `trec_eval` walks the recall levels backward without sorting them (causing out-of-order lists to return wrong scores).
- **Duplicate rejection:** `te-rust` rejects duplicate recall levels in `11pt_avg` (e.g., `11pt_avg.0.5,0.5`) rather than silently double-weighting the repeated point.

### Float Parameter Domains & Finiteness
- Non-finite floats (`nan`, `inf`, `-inf`, and overflowing literals like `1e400`) are rejected in every float-parameterized measure.
- Domain bounds are enforced:
  - `iprec_at_recall` and `11pt_avg`: recall levels must lie in $[0.0, 1.0]$.
  - `Rprec_mult` and `Rprec_mult_avgjg`: multipliers must be strictly positive ($> 0.0$).
  - `set_F`: weight $x$ must satisfy $x \ge 0.0$ ($x = 0$ is legal and equals `set_P`).
  - `rbp` and `rbp_resid`: persistence $p$ must lie in $[0.0, 1.0)$ ($p = 0$ is legal; $p = 1$ is rejected).
- Exact duplicate float cutoffs (including `-0.0` alongside `0.0`) are rejected. Distinct floats that format to the same two-decimal label (e.g., `Rprec_mult.0.999,1.001`) remain allowed, matching C `trec_eval`.

### Gain Mappings (`G`, `ndcg`, `ndcg_rel`, `Rndcg`, `ndcg_p`, `rbp`, `--global-gains`)
- Every entry in a gain map must have the form `<int>=<finite float>`.
- Missing `=`, non-integer relevance levels, non-numeric or non-finite gains, and duplicate relevance levels (including equivalent integer spellings such as `1=2,01=3`) are rejected instead of being skipped, coerced to `0.0`, or overwritten.

### `rbp` and `rbp_resid` Parameter Keys
- `rbp` accepts `p=<float>` and optional `<int>=<float>` gain overrides; any other key name or duplicate `p` is rejected.
- `rbp_resid` accepts only `p=<float>`; unknown keys (including numeric gain keys, which `rbp_resid` does not use) and duplicate `p` are rejected.

### Integer Rank Cutoffs & Parameterless Measures
- Rank cutoffs (`P`, `recall`, `ndcg_cut`, `map_cut`, `relative_P`, `success`, `unj`, `P_avgjg`) must be positive integers in `[1, i64::MAX]` (`9223372036854775807`), unique, and non-empty. C `trec_eval` 10.0 clamped values above `LONG_MAX` instead of rejecting them; `te-rust` matches C `trec_eval` `main` by rejecting them with exit status `2`.
- `relstring` accepts length `0` (producing `''`) up to `i64::MAX` (not recommended).
- Passing parameters to a parameterless measure (e.g., `-m map.5`) is rejected with exit status `2`.

---

## 2. Additional Measure Specification Syntax

In addition to standard `trec_eval` `.params` syntax, `te-rust` accepts:

- `@` cutoff aliases: `ndcg@10`, `map@100`, `P@5,10`, `recall@10`, etc.
- Measure name aliases: `mrr` / `rr` for `recip_rank`, `precision` for `P`.
- Underscore cutoff shorthands: `P_5,10`, `ndcg_cut_10`, `map_cut_100`, `recall_10`, `success_5`, `unj_10`, `P_avgjg_10`.

Empty parameter suffixes (`P.`, `P_`, `P@`) are rejected with exit status `1`.

---

## 3. Input Parsing Guards & Comment Handling

- **Invalid and non-finite similarity scores:** Run files (and Python in-memory run inputs) containing non-numeric tokens (such as `abc`), trailing non-numeric characters (such as `1.5x`), `NaN`, or `Infinity` (`inf`, `-inf`, or overflowing literals like `1e400`) are rejected with an error rather than coerced to `0.0`, partially parsed with a warning, or propagated into unstable rankings.
- **Relevance grade bounds:** Qrels relevance levels are validated to fall within `[-1,000,000, 1,000,000]` to guard against unbounded memory allocation when indexing relevance-level histograms.
- **Comment indentation:** Lines whose first non-whitespace character is `#` are treated as comments in qrels and run files (C `trec_eval` requires `#` in column 0).

---

## 4. Granular Help Output

In C `trec_eval`, `-h` prints full descriptions for every measure in a single long dump. In `te-rust`:

- `-h` / `--help` prints concise CLI usage and option flags.
- `--help-measures` lists available measures with their lifecycle status and short descriptions.
- `--help-measure <name>` prints the detailed explanation and parameter usage for a single measure.

---

## 5. Measure Lifecycle Status

Measures in `te-rust` carry a lifecycle status (`Production`, `Experimental`, or `Obsolete`) displayed by `--help-measures` and `--help-measure`.

- Experimental measures (currently `yaap`) are excluded from predefined measure groups (`official`, `all_trec`, `set`, `qrels_jg`), while remaining explicitly requestable with `-m <name>`.
