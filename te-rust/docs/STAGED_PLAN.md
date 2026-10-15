# `te-rust` Staged Implementation Plan

This document defines the structured, three-stage development plan for `te-rust`, transitioning from core parsing and standard evaluation up to specialized parametric measures and preference-based evaluations.

---

## Stage 1: Core Parsing & Standard Evaluation (`all_trec`)

The primary objective of Stage 1 is to build the core evaluation pipeline, support standard inputs (standard qrels and run results), and implement the `all_trec` standard metric set to verify perfect numerical parity with C `trec_eval` under standard quicktests.

```mermaid
graph TD
    A[Parsers: Standard Qrels & Runs] --> B[Alignment Engine: QueryEvalState]
    B --> C[Metric Registry: all_trec Subset]
    C --> D[CLI Harness: File Loading & Execution]
    D --> E[Regression Verification: tests/regression.rs]
```

### 1.1 Project Structure Setup
*   Initialize the `te-rust` directory with a standard workspace structure.
*   Configure `Cargo.toml` with release optimizations (e.g., Link-Time Optimization (LTO) and codegen-units) to match C execution speeds.

### 1.2 Core Input Parsers (`src/io/parsers.rs`)
*   **Standard Run Parser**: Whitespace-separated fields (`qid  iter  docno  rank  sim  run_id`).
*   **Standard Qrels Parser**: Whitespace-separated fields (`qid  iter  docno  rel`).
*   **Safety Guards**: 
    *   Strict non-finite float rejection (`NaN`, `Infinity`) for similarity scores.
    *   Generous relevance score boundary validation (`-1,000,000` to `1,000,000`) to prevent allocation exploits.
    *   Indentation-agnostic `#` comment stripping (trimming leading spaces).

### 1.3 Aligned Evaluation Engine (`src/eval/alignment.rs`)
*   **Tie-Breaking sorting**: Sort results descending by `sim` (primary), and **lexicographically descending** by `docno` (secondary) to match C tie-breaking exactly.
*   **Defensive Allocation**: Populate `QueryEvalState::rel_levels` pre-allocated and initialized with zeros to `max(max_rel + 1, config.relevance_level + 1)` size.
*   **Complete Set Average (`-c`)**: Implement native empty execution states (`Vec::new()`) for topics missing from the run results file under `-c`, ensuring proper default score evaluation without overrides.

### 1.4 The `all_trec` Metric Registry (`src/metrics/`)
Implement the `Measure` trait and the following core metrics (the standard `all_trec` set):
*   **Count measures**: `num_ret`, `num_rel`, `num_rel_ret`.
*   **Mean Average Precision (`map`)**.
*   **Precision at cutoffs (`P_cut`)**: Precision at fixed ranks (5, 10, 15, 20, 30, 100, 200, 500, 1000).
*   **R-Precision (`Rprec`)**.
*   **Reciprocal Rank (`recip_rank`)**.
*   **Binary Preference (`bpref`)**: Binary preference computation using bounds-checked counts.
*   **nDCG at cutoffs (`ndcg_cut`)**: Discounted Cumulative Gain utilizing `f64::total_cmp` for deterministic ranking and division-by-zero guards.

### 1.5 CLI Driver & Averages Accumulator (`src/main.rs`)
*   Load files, invoke alignment, and execute metrics.
*   Accumulate per-query totals and calculate final summary averages.
*   Output scores in standard relational format (`measure  qid  value`).

### 1.6 Verification Harness (`tests/regression.rs`)
*   Copy standard test files (`qrels.test`, `results.test`, `out.test`, `out.test.a`, `out.test.aq`, `out.test.aqc`) from `trec_eval` to `tests/test_data/`.
*   Implement the regression test runner using precompiled binaries via `env!("CARGO_BIN_EXE_te-rust")`.
*   Execute key-by-key relational triple assertions with a float epsilon of `1e-4` to secure perfect numerical equivalence.

---

## Stage 2: Help Granularity, Stock Groups, and Parametric Measures [Completed]

Stage 2 extends the core pipeline to support dynamic user assistance, standard stock groups (nicknames), dynamic parametric metrics, and complete standard command-line options.

### 2.1 Help Granularity, Sane CLI Defaults, and Flag Expansion (Issue 21) [Completed]
*   **Granular Help System [Completed]**:
    *   Extend `Measure` trait to include `fn short_description(&self) -> &'static str`.
    *   Implement `--help-measures` which lists only available measure names alongside their short descriptions and exits immediately.
    *   Implement `--help-measure <name>` which retrieves the requested measure and prints its full description via `explanation()` and exits immediately.
*   **Sane CLI Defaults [Completed]**:
    *   If no `-m` flag is supplied on the command line, `te-rust` defaults to executing and reporting the `"official"` stock group of measures.
*   **CLI flag parity [Completed]**: Complete support for `-N <num>` (collection size) and map its value appropriately.

### 2.2 Predefined Groups & Dynamic Parameter Parsing [Completed]
*   **Stock Group Nicknames [Completed]**:
    *   Define group mappings for `official`, `all_trec`, and `set`.
    *   When an argument to `-m` matches a group nickname, dynamically expand it to include all corresponding individual measures.
*   **Dynamic Parameter Parsing [Completed]**:
    *   Refactor metrics registry to support parsing parameter cutoffs dynamically from period-delimited parameter strings (e.g. `P.5,10` or `ndcg_cut.10`).
    *   Allow multiple independent `-m` flags on the command line, expanding and merging all specified measures.

### 2.3 Specialized Standard Metrics and Non-Standard Formats [Completed]
*   **Additional measures [Completed]**: Implement `success`, 11-point average precision (`11pt_avg`), parametric standard `recall`, cost-weighted `utility`, and relevance string visualizer `relstring`.
*   Ensure that any incompatibilities between requested metrics (such as pairwise metrics when standard qrels are used) are checked and aborted during metadata evaluation.

### 2.4 Verification and Regression Suite Expansion [Completed]
*   Extend regression tests in `tests/regression.rs` to run and verify dynamic parametric quicktests (`test_meas_params`, `test_relstring_relevance_level`).

### 2.5 Measure-Specific Edge Case Unit Tests [Completed]
*   **Locality of Tests**: Implement `#[cfg(test)] mod tests` submodules inside each individual metric file to keep tests close to their measure's mathematical code.
*   **Shared Test Generator**: Implement a unified, mock-aligned `QueryEvalState` builder in `src/metrics/common.rs` under `#[cfg(test)]` to prevent repetitive setup code across the 15+ metrics.
*   **Edge Case Coverage**: Verify standard metrics across standard edge cases (empty rankings, zero retrieved documents, no relevant documents retrieved, single relevant retrieved, and numerical rounding bounds) using discrete, descriptively named unit test functions.



---

## Stage 3: Preference-Based Evaluation & Comparative Verification

Stage 3 implements the complete preference-based evaluation architecture (`trec_prefs`), verifying partial-order zones and transitive relations.

### 3.1 Preference Input Parsers
*   **Prefs Parser**: Parses explicit pairwise preferences (`qid  docno_A  docno_B  JSG_id`).
*   **Qrels-Prefs/Qrels-JG Parsers**: Map qrels records to preference counts.

### 3.2 Preference Alignment & Dual Transitive Closures
*   Assign deterministic internal ranks (`0..num_judged`) grouping retrieved items first.
*   Implement both transitive closure methods in the engine:
    *   **Strategy A (Naive Matrix Exponentiation)**: Exact behavioral port of C's multiplication loop for strict verification.
    *   **Strategy B (Bit-Parallel Warshall)**: High-performance $O(N^3/64)$ production optimizer.
*   During test execution, run both strategies in parallel and assert matrix equivalence to mathematically prove Strategy B's safety.

### 3.3 Preference Metrics & Final Validation
*   Implement `prefs_num_prefs`, `prefs_simp`, `prefs_pair`, `prefs_avgjg`, etc.
*   Run final quicktests (`test_all_prefs`, `test_all_prefs_comments`, `test_qrels_jg`) to verify complete parity across all evaluation formats.
