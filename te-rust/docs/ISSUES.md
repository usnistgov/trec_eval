# `te-rust` Design and Implementation Issues

This document tracks active design questions, structural decisions, and implementation tasks for the `te-rust` project. Issues are categorized by type and updated as we progress.

---

## Active Issues

(None currently active)


---

## Resolved Issues

### 28. Predefined Measure Group Definitions & Output Parity
*   **Type**: bug
*   **Status**: Resolved
*   **Description**: Aligned predefined measure groups (`all_trec`, `official`, `set`, `qrels_jg`) in `metrics/registry.rs` with C `trec_eval`'s canonical definitions from `measures.c`. Implemented `NumQMeasure` in `metrics/num_q.rs` (reporting query count in summary `all`, suppressed from query-level output matching C `te_print_single_meas_empty`). Implemented missing set measures `SetPMeasure` (`set_P`) and `SetRecallMeasure` (`set_recall`). Fixed `RunIdMeasure` query-level suppression (`is_query_enabled = false`). Individual `-m` command-line flags maintain user-specified insertion order while predefined group expansions produce canonical C `trec_eval` ordering. Verified 0-diff parity against live C `trec_eval` on standard and JG test suites.

### 22. Confidence intervals
*   **Type**: feature
*   **Status**: Resolved
*   **Description**: Implemented percentile bootstrap confidence intervals for summary metrics over evaluated queries, with zero heavy external dependencies (built on a lightweight, seedable SplitMix64 PRNG). Resampling core is implemented in `src/eval/bootstrap.rs` supporting both standard arithmetic mean aggregation and geometric-mean log transforms (`gm_map`, `gm_bpref`), serving as the shared foundation for both CLI and the future Python bindings. Enabled via `-C / --ci` which preserves the strict 3-column relational format (`<measure>_ci_lower` and `<measure>_ci_upper` summary rows) for compatibility with Unix and TREC relational tools, while human-readable bracketed output `[lower, upper]` is supported via `--ci-pretty`. Configurable via `--ci-alpha` (default 0.05), `--ci-samples` (default 1000), and `--seed <u64>`. Comprehensive unit tests and integration tests added.

### 13. Enforcing Uniform Boundary Testing across all Metrics
*   **Type**: Design
*   **Status**: Resolved
*   **Description**: Resolved to model boundary properties as declarative `Invariant` enum items (`Finite`, `NonNegative`, `EmptyRankingIsZero`, `ZeroRelevanceIsZero`, `UnitInterval`), declared directly on each measure via `Measure::invariants(&self) -> &'static [Invariant]`. This serves as self-documentation on the trait, while a centralized test-time checking loop (`metrics::invariants::checking`) iterates the entire measure registry and tests all 33 measures against their declared invariants across multiple boundary scenarios (empty rankings, zero-relevance topics, unjudged topics). Divergent categories cleanly override their declared invariant bundles (e.g. `COUNTS` for count measures, `SIGNED` for utility, `FINITE_ONLY` for log-transformed measures like `gm_map`/`gm_bpref`/`yaap`, `NONE` for string tags). Replaced ~370 lines of boilerplate `test_*_empty_ranking` and `test_*_zero_relevance` tests across individual metric files while preserving unique, measure-specific test scenarios.

### 24. Measures can be production, experimental, or obsolete
*   **Type**: feature
*   **Status**: Resolved
*   **Description**: Added a `MeasureStatus` enum (`Production`, `Experimental`, `Obsolete`) carried per measure in the registry `MeasureSpec`. This is a new concept (C `trec_eval` had no measure statuses). Assigned statuses: `G`, `yaap`, `binG` are Experimental; everything else (including `rbp`/`rbp_resid`) is Production; no measures are currently Obsolete (variant retained for future use). Experimental measures are excluded from predefined groups (`official`/`set`/`all_trec`) — `G` was removed from `all_trec` — but remain requestable explicitly via `-m`. Status is shown in help output (#23). Added a registry invariant test that no group contains a non-production measure, and extended the regression suite to exercise `G` and `binG` against C (previously uncovered, since no test used `-m all_trec`).

### 23. Help text needs to be more helpful
*   **Type**: bug
*   **Status**: Resolved
*   **Description**: Added a `usage` field to `MeasureSpec` describing how to request each measure and the parameters/defaults it accepts (rank cutoffs, recall levels, gain mappings, utility coefficients, F beta, RBP persistence). `--help-measure <name>` now prints the measure name, its status, the explanation, and a `Usage:  -m <name>[.params]` line; `--help-measures` gained a Status column flagging experimental/obsolete measures. Added `MeasureStatus::label()` and a registry test asserting each non-empty usage line begins with its measure name.

### 26. Cutoffs in measure initialization
*   **Type**: bug
*   **Status**: Resolved
*   **Description**: Added shared parameter parsers in `metrics/common.rs` (`parse_int_cutoffs`, `parse_float_cutoffs`, `parse_key_values`, plus `MeasureParseError`). Every parameterized measure now parses its cutoffs/coefficients through these helpers via its registry factory, eliminating the ~15 copy-pasted parse loops that previously lived in `main.rs`. Resolved together with #25.

### 25. main.rs is a mess
*   **Type**: bug
*   **Status**: Resolved
*   **Description**: Introduced a single measure registry in `metrics/registry.rs` (`MeasureSpec { name, status, factory }`), the direct analog of C `trec_eval`'s `te_trec_measures[]` table. Groups (`official`, `set`, `all_trec`) are name lists resolved against the table via `expand_group()`, and `resolve_measures()` turns requested `root[.params]` arguments into constructed measures with errors that name the offending argument. `main.rs` was cut over to call the registry for both `-m` handling and help output, dropping from ~690 to 271 lines; the ~380-line match, the duplicate help-flags list, and the now-dead `get_measures_for_all_trec()` were removed. All measure declaration and option parsing is delegated to the measure/registry code. Verified behavior-preserving by the full unit + regression suites.

### 27. Standard Uncut NDCG and Cutoff Behavior Alignment
*   **Type**: Bug
*   **Status**: Resolved
*   **Description**: Audited and confirmed all 15 core measures' cutoff behavior. Discovered that the standard uncut `ndcg` measure (which evaluates dynamically to the end of both the retrieved ranking and the ideal relevance ranking) was missing from the registry. Implemented `ndcg` in `te-rust/src/metrics/ndcg.rs`, registered it in `metrics/mod.rs` and `main.rs`, and updated the regression tests to verify that both C and Rust align exactly on all evaluation runs.

### 21. Simplified Help Output with Per-Measure Granularity
*   **Type**: Design / Feature
*   **Status**: Resolved
*   **Description**: In standard `trec_eval`, running `-h` prints a massive wall of text containing detailed documentation of every single metric, which is overwhelming. For `te-rust`, we designed a granular, interactive help system:
    1. A generic help switch (`--help-measures`) lists only the names of the available measures and their brief descriptions.
    2. A specific option (`--help-measure <name>`) prints the detailed documentation and explanation for only that requested measure.
    3. The detailed documentation/explanation is defined directly in the source code of the individual `Measure` implementation via its `explanation()` and `short_description()` trait methods.

### 1. Unified vs. Hierarchical Flat Arrays (De-duplication of Query IDs)
*   **Type**: Design
*   **Status**: Resolved (Option B)
*   **Description**: Resolved to use hierarchical structs (`RunQuery`, `QrelsQuery`) to group records by `qid`, avoiding duplicate query ID string allocations and naturally aligning with C's query grouping.

### 2. Stream-based Reading and I/O Genericity
*   **Type**: Design
*   **Status**: Resolved (Option B)
*   **Description**: Decided to use standard `std::io::BufRead` trait for parser inputs. This decouples the core parsing logic from physical file opening, allows simple unit testing using in-memory byte streams, and directly enables parsing from Standard Input (`stdin`).

### 3. Immediate Abort vs. Collected Errors during Parsing
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Decided to abort parsing immediately on the first syntax error. To compensate for not collecting errors, we will heavily prioritize highly detailed and helpful error diagnostics (including line numbers, raw line contents, expected format, and the reason for parsing failure).

### 4. Sorting non-Ord floats (`f64`) in Rust
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to explicitly validate during parsing that all similarity scores are valid finite numbers (rejecting `NaN` and `Infinity` with an error). This guarantees that `.partial_cmp().unwrap()` is entirely safe and infallible for sorting without needing third-party wrapper dependencies.

### 5. Configuration Metadata in Comments
*   **Type**: Design/Future Feature
*   **Status**: Resolved (Option A - TOML Blocks)
*   **Description**: File comments starting with `#` can contain inline configuration metadata formatted as a TOML block. The ingestion phase strips the leading `#` (and up to one space) from comment lines and saves them to a flat vector of comments. A post-parser metadata resolver scans these cleaned comments for a `[te-rust.config]` block, deserializes it using standard TOML libraries, and merges it into the execution parameters with a strict hierarchy of precedence: Default Settings < File-Specified Config < User CLI Arguments.


### 6. Double-Precision Floating-Point Compatibility (Rust vs C)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to use standard IEEE 754 64-bit float math (`f64`) natively, which maps to identical CPU-level instructions in modern compiled targets. If tiny floating-point rounding variations or transcendental libm math changes emerge during differential integration testing, we will first verify the Rust metric math for 100% correct implementation. Upon confirmation of correctness, we will either update our local quicktest expected output files to match Rust's highly precise calculations, or we will enhance our regression test runner to parse the tables and check scores with a numeric epsilon (e.g., `1e-5`) rather than checking strict string equality.

### 7. Evaluation Corner Cases (Empty Runs, Zero-Relevance Topics)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to define mathematically safe default values individually for each metric under boundary cases (such as empty rankings, zero relevant documents, etc.), precisely aligning with the legacy defaults established by `trec_eval` (typically `0.0`, but varies where alternative baseline formulations exist). All calculators will defensively check denominators to prevent any generation of `NaN` or `Infinity` values, and our centralized boundary test suite will automatically run these edge cases against every metric to verify absolute safety and correctness.


### 8. Printing and Formatting Types for Measures
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to define a standard `ValueFormat` enum containing `Float`, `Integer`, `Str`, and `QuotedStr` variants. Each implementation of the `Measure` trait is required to expose its desired format via a trait method (e.g., `fn format(&self) -> ValueFormat`). The printing presentation layer will automatically format outputs accordingly, completely decoupling calculation code from stdout formatting.

### 9. Representation and Printing of String-Valued Measures (e.g., `runid`, `relstring`)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to introduce a standard `MetricValue` enum containing `Float(f64)`, `Integer(i64)`, and `Str(String)` variants. The `calc` method of our `Measure` trait returns this structured enum. This allows string-valued metrics (like `runid` and `relstring`) to be calculated thread-safely and returned directly to the harness without any dangerous C-style pointer-casting or global variable hacks. The print formatter formats them using `ValueFormat`, including support for wrapping string sequences in single quotes (as required for backwards-compatible `relstring` outputs).


### 10. Metric Compatibility with Ground-Truth Formats via Traits
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to use a single unified `Measure` trait exposing an `eval_type(&self) -> EvaluationType` method (returning `Standard`, `Preferences`, or `JudgmentGroups`). To optimize execution, the validation pass is performed **immediately after the ground-truth (qrels) file is loaded and parsed**, since the ground-truth format dictates the entire evaluation paradigm. If any requested measure is incompatible with the parsed ground-truth format, the harness aborts instantly with a clean diagnostic error before wasting I/O or memory opening and parsing the potentially multi-gigabyte run results file.


### 11. Regression Testing Harness in Rust
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to implement the regression check suite directly inside Rust's standard integration testing framework as `tests/regression.rs`. The test runner copies/references the regression files inside the Cargo package, compiles the `te-rust` binary, and executes it as a child process using `std::process::Command` across the 13 quicktest scenarios. It captures the stdout and compares it with the expected C outputs (out.*) line-by-line, providing highly detailed diff context on mismatch while integrating flawlessly with `cargo test` for local development and CI pipelines.


### 12. Avoiding Boundary Test Duplication via Shared Declarative Macros
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to create a standard `test_measure_boundaries!` macro in our test common module, coupled with a helper generator `make_mock_state`. This enables metrics to cleanly declare their own specific boundary condition matrices in a highly compact, data-driven way, completely eliminating duplicate mock state configuration and assertion loops.


### 14. Stateless `Measure` Trait vs. Parameter Mutation
*   **Type**: Design
*   **Status**: Resolved (Stateless Factory Pattern)
*   **Description**: Resolved to refactor the `Measure` trait to be completely stateless and thread-safe. Instead of calling a mutable `init(&self)` method on a single pre-allocated metric instance, individual metric instances are instantiated up front with their specific parsed parameters (e.g. cutoffs) as immutable fields. The trait's `sub_metrics(&self)` and `initial_values(&self)` methods expose these pre-computed configurations to the evaluation harness, enabling efficient, lock-free parallel execution.

### 20. Infallible Float Sorting and Determinism
*   **Type**: Design
*   **Status**: Resolved (total_cmp & Measure-Dependent guards)
*   **Description**: Resolved to incorporate `f64::total_cmp` for high-performance, branchless, and crash-safe document sorting within the alignment engine, coupled with strict fail-fast validation in the parser to reject `NaN` and `Infinity` float inputs. Furthermore, recognized that "correct fallback score" under undefined states is measure-dependent (e.g., 0.0 vs utility offsets); resolved that each individual metric `Measure` implementation must defensively evaluate its own specific mathematical boundaries and return its designated standard fallback rather than propagating raw floating-point `NaN` or `Infinity` values.

### 18. Asymmetric Comment Handling
*   **Type**: Design
*   **Status**: Resolved (Option B - Relaxed Quality-of-Life)
*   **Description**: Resolved to adopt Option B (relaxed quality-of-life parsing), which trims leading whitespace before checking for `#` comment characters. Since comment support is a highly recent addition in `trec_eval`, backward-compatibility of files from `te-rust` to old legacy C versions is not a primary concern. Allowing leading spaces before comments provides a much better and more forgiving user experience for manual file editing, avoiding fatal parsing failures on minor indentation variations.

### 17. Defensive `rel_levels` Bounds Sizing
*   **Type**: Design
*   **Status**: Resolved (Defensive Vector Sizing & Safe Getters)
*   **Description**: Resolved to implement a three-layer boundary protection design for relevance counts. First, to support modern research on pairwise tournament and dense-scale preference evaluations while preventing OOM denial-of-service vulnerabilities, the lexical parser strictly validates that parsed relevance levels fall within a safe, generous range of `-1,000,000` to `1,000,000` (max 8MB memory allocation). Second, the alignment engine defensively sizes the `rel_levels` flat vector to `max(max_rel + 1, config.relevance_level + 1)`. Third, all metrics are mandated to retrieve counts from `rel_levels` using infallible, bounds-checked getters (e.g. `.get(j).copied().unwrap_or(0)`), guaranteeing 100% crash-free execution.

### 16. The `bogus_ranking` Ingestion Hack
*   **Type**: Design
*   **Status**: Resolved (Native Empty State Evaluation)
*   **Description**: Resolved to natively evaluate missing queries under complete set evaluation (`-c`) using empty query execution states (`Vec::new()`), completely eliminating the internal `bogus_ranking` and the subsequent fragile, hardcoded post-evaluation override loops. Because our metric architecture requires measures to self-contain their boundary conditions defensively (as resolved in Issue 20), an empty state natively evaluates to correct baseline totals and averages across all standard, utility, and complex metrics. Hand-inserted dummy lines in run files are still parsed as normal, maintaining 100% parity with C `trec_eval` under all execution states.

### 15. Lack of Design for Preference-Based Evaluations
*   **Type**: Design
*   **Status**: Resolved (Full Preference Mapping & Dual-Strategy Testing)
*   **Description**: Resolved to specify a comprehensive, type-safe Rust representation for the entire preference evaluation logic, completely defining the previously empty `PrefsEvalState`, `JudgmentGroup`, and `EquivalenceClass` structures in `EVAL_DESIGN.md`. Documented the alignment engine's exact internal doc ranking assignment (0..num_judged), Equivalence Class sorting, and partial-order preference matrix construction. To prevent regression bugs and guarantee 100% behavioral alignment with C's complex `form_prefs_counts.c`, we resolved to implement two independent transitive closure algorithms: a naive C-style iterative matrix exponentiation (the reference oracle) and a bit-parallel Warshall's algorithm (production optimizer). The evaluation engine will run both strategies in parallel during development and compare resulting matrices. This guarantees that we verify the production optimizer's safety under all imaginable dataset conditions before disabling the naive reference in production. Fully mapped the five combinatorial topological count areas (A1–A5) for both layouts.

### 19. Fragile Testing Harness: `cargo run` and Exact String Comparisons
*   **Type**: Testing
*   **Status**: Resolved (CARGO_BIN_EXE and Relational Epsilon Parser)
*   **Description**: Resolved to rebuild the integration testing suite around modern Rust testing best practices. First, replaced recursive `cargo run` spawning with standard pre-compiled binary execution using `env!("CARGO_BIN_EXE_te-rust")`, bypassing the cargo locks deadlock vector and enabling 100% parallel integration testing execution. Second, replaced fragile exact-line string matching with a structured relational regression parser that reads whitespace-separated `trec_eval` output triples (`measure qid value`). Triples are compared key-by-key: numeric values are evaluated within a defensive float tolerance epsilon (`1e-4` or `10^-4`), while text columns (e.g. `run_id`) fall back to formatting-agnostic string matching. This guarantees regression safety while rendering the tests robust to spacing and platform-specific floating-point representation details.
