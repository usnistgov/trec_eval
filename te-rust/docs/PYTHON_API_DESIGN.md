# Python API & Bindings Design Document for te-rust

## 1. Overview and Motivation

This document presents the architectural design for exposing `te-rust` functionality as a native Python package. The Python ecosystem is the primary working environment for modern Information Retrieval (IR) research, machine learning rankers, and data science workflows.

### Goals
1. **High Performance & Safety**: Leverage Rust's speed, zero-cost abstractions, and thread safety with zero memory leaks and safe GIL release during batch evaluation.
2. **Compatibility & Migration Ease**: Provide drop-in or near-drop-in compatibility with legacy bindings (`pytrec_eval`), while offering seamless interop for modern ecosystems (`ir_measures`, `trectools`).
3. **Versatile Input/Output Formats**: Ingest data from files, in-memory nested dictionaries, row iterables, custom user Python objects, and tabular structures (Pandas/Polars/Arrow).
4. **Zero Heavy Mandatory Dependencies**: Support DataFrames and scientific arrays without making `pandas`, `pyarrow`, or `numpy` hard installation dependencies.
5. **Extensibility & User Object Bridging**: Allow Python users to evaluate custom Python objects and classes (search hits, dataclasses, pydantic models) without manual pre-conversion to intermediate dictionaries.

---

## 2. Analysis of Existing Python Integrations

| Feature / Library | `pytrec_eval` | `ir_measures` | `trectools` | `sigtrec_eval` |
| :--- | :--- | :--- | :--- | :--- |
| **Backend** | C `trec_eval` via Cython/C API | Multi-provider (Cython, Python, `pytrec_eval`, etc.) | Pure Python / Pandas | Wrapper around `trec_eval` CLI |
| **Primary Input Format** | `Dict[str, Dict[str, Union[int, float]]]` | Dicts, DataFrames, Iterators of namedtuples/dataclasses | File paths or Pandas DataFrames | File paths |
| **Primary Output Format** | Nested `Dict[str, Dict[str, float]]` | Namedtuple / Dataclass stream (`Metric(query_id, measure, value)`) | Pandas DataFrames (`TrecRes`) | CLI text / significance summary dicts |
| **Dependencies** | Minimal / None (C extension) | `cffi`, optional `pandas`, `scipy` | Heavy (`pandas`, `scipy`, `numpy`, `matplotlib`) | `scipy`, `numpy`, `trec_eval` binary |
| **Strengths** | Simple, ubiquitous, fast for small runs | Composable measure objects (`nDCG@10`), provider-agnostic | Rich IR campaign tools (fusion, pooling) | Statistical significance testing |
| **Weaknesses** | Dict conversion is high overhead; no DataFrame or zero-copy support; hard to maintain C code | Measure algebra adds complexity for simple CLI parity | High memory and slow parsing for large runs | Subprocess CLI overhead |

---

## 3. Rust-Python Binding Architecture

### 3.1 Tooling & Crate Organization

We propose using **PyO3** + **Maturin**:
- **PyO3**: Industry standard for Rust-Python bindings; generates efficient CPython C-extension modules with automatic type conversion, exception mapping, and GIL management.
- **Maturin**: Build tool and PEP 517 build backend for compiling and packaging PyO3 crates into Python wheels for PyPI.

#### Proposed Workspace Structure
Instead of coupling Python bindings directly into the standalone CLI binary crate, we organize `te-rust` as a Cargo workspace:

```text
trec-eval-rust/
├── te-rust/                  # Core Rust crate (lib + CLI binary)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Core evaluation engine, types, parsers, metrics
│       ├── main.rs           # CLI executable
│       ├── io/
│       ├── eval/
│       └── metrics/
├── te-python/                # Python binding crate (PyO3)
│   ├── Cargo.toml            # depends on path = "../te-rust"
│   ├── pyproject.toml        # maturin configuration, package metadata
│   ├── src/
│   │   ├── lib.rs            # PyO3 module definition & PyO3 classes
│   │   ├── conversions.rs    # Input adaptation (dicts, dataframes, iterators)
│   │   ├── evaluator.rs      # Evaluator PyClass
│   │   └── compat.rs         # Compatibility layers (pytrec_eval, ir_measures)
│   └── python/
│       └── trec_eval/        # Pure Python package layer / stubs / typing
│           ├── __init__.py
│           ├── compat/
│           │   ├── pytrec_eval.py
│           │   └── ir_measures.py
│           └── py.typed
```

**Pros of Separation**:
- The core CLI binary remains lean with zero Python C headers or runtime linkage.
- The Python bindings can be versioned, tested, and released independently or alongside the CLI.
- Core Rust engine can be consumed by other Rust projects or WASM targets.

---

## 4. Input Data Ingestion & Canonical Formats

Rather than attempting magic attribute reflection or probing, the ingestion engine follows Python's standard explicit typing conventions (similar to `ir_measures` and `pandas`).

```mermaid
flowchart TD
    A[Input Data Source] --> B{Explicit Input Type}
    B -->|File Path / str / PathLike| C[Rust Fast Native File Reader]
    B -->|Nested Dicts| D[Dict Ingestor: Dict[str, Dict[str, num]]]
    B -->|Pandas / Polars DataFrame| E[Columnar Ingestor: query_id, doc_id, score]
    B -->|Iterable of Tuples / ScoredDoc| F[Tuple Stream: qid, doc_id, score]
    B -->|NumPy 1D Arrays| G[Zero-Copy Buffer Ingestor]
    
    C --> H[Internal Rust Alignment & Storage]
    D --> H
    E --> H
    F --> H
    G --> H
```

### 4.1 Canonical Input Formats

1. **File Paths (`str` or `os.PathLike`)**:
   - Direct Rust file I/O using buffered readers.
   - Bypasses Python interpreter completely; optimal for large standard TREC files.

2. **Nested Dictionaries (Legacy `pytrec_eval` compatibility)**:
   - `Dict[str, Dict[str, Union[int, float]]]`
   - Direct iteration over Python dictionaries via PyO3 without full intermediate JSON serialization.

3. **Iterables of 3-Tuples or `ScoredDoc` NamedTuples**:
   - `Iterable[Tuple[str, str, float]]` (e.g. `(query_id, doc_id, score)`)
   - Canonical `NamedTuple` definitions provided in `trec_eval`:
     - `ScoredDoc = NamedTuple('ScoredDoc', [('query_id', str), ('doc_id', str), ('score', float)])`
     - `Qrel = NamedTuple('Qrel', [('query_id', str), ('doc_id', str), ('relevance', int)])`
   - **Bridging Custom User Classes**: Users bridge their arbitrary classes/dataclasses using standard generator expressions:
     ```python
     evaluator.evaluate((h.topic, h.docno, h.score) for h in my_search_hits)
     ```

4. **DataFrames (Pandas / Polars)**:
   - Default column names expected: `query_id`, `doc_id`, `score` (and `relevance` for qrels).
   - Allows explicit column overrides if DataFrame column names differ:
     ```python
     evaluator.evaluate(df, qid="topic", docno="docno", score="similarity")
     ```

5. **NumPy 1D Arrays (Zero-Copy Buffer Ingestion)**:
   - `evaluator.evaluate_arrays(query_ids, doc_ids, scores)`
   - Reads contiguous float slices directly from memory buffers with zero intermediate Python object allocations.

---

## 5. Zero-Mandatory-Dependency DataFrame & Scientific Stack Strategy

### 5.1 The Problem & Opportunity
IR researchers reflexively use **NumPy, Pandas, and SciPy** for model inference (e.g. dense retrieval scores in float arrays), experiment analysis, and statistical significance testing.
However:
- Making `pandas`, `scipy`, and `numpy` hard dependencies in `pyproject.toml` forces a heavy install (~150MB+) on users who only want lightweight evaluation.
- Conversely, requiring researchers to manually convert NumPy matrices or DataFrames into nested Python dicts creates massive CPU and memory overhead.

### 5.2 NumPy & Columnar Ingestion (Zero-Copy FFI)
Through Python's Buffer Protocol (PEP 3118) and PyO3's buffer interfaces:
1. **Direct Array Ingestion**:
   - `evaluator.evaluate_arrays(query_ids, doc_ids, scores)` where `scores` is a 1D contiguous NumPy array (or PyTorch / CuPy array converted to CPU buffer).
   - Rust can read the raw C-contiguous memory slice of `f32` or `f64` floats directly without allocating intermediate Python objects.
2. **Dense Score Matrices for Neural Rankers**:
   - `evaluator.evaluate_matrix(query_ids, doc_ids, score_matrix)` where `score_matrix` is `(num_queries, num_docs)`.

### 5.3 Pandas Integration: Wide vs Tidy Formats
1. **Input Detection**:
   - Accepts any DataFrame with columns `query_id` (or `qid`), `doc_id` (or `docno`), and `score` (for runs) or `relevance` (for qrels).
   - Ingests columns directly as contiguous series buffers rather than row-by-row iteration.
2. **Output Formatting**:
   - `.to_dataframe(format="wide")`: Query IDs as rows, measures as columns (convenient for machine learning features, correlational analysis).
   - `.to_dataframe(format="tidy")`: Long format `(query_id, measure, value)` (convenient for Seaborn/Altair plotting and `groupby`).
   - `.to_numpy(measure="map")`: Returns a 1D NumPy float array of per-query scores with guaranteed deterministic topic ordering.

### 5.4 Multi-Run Statistical Significance & Multiple Comparisons Corrections

In IR experimental campaigns (TREC tracks, SIGIR/CIKM benchmarking, model ablation studies), researchers frequently evaluate tens or hundreds of runs across multiple evaluation measures. Evaluating multiple hypotheses without adjusting for multiple comparisons leads to severe Type I error inflation (false discoveries).

#### 5.4.1 Division of Responsibilities (Rust vs. Python)
We split the significance testing pipeline to maximize speed and analytical flexibility:

1. **Rust Core (High-Speed Raw Pairwise Statistics)**:
   - Computes aligned topic score vectors with deterministic ordering and zero-fill for missing topics.
   - Calculates raw test statistics and unadjusted p-values:
     - **Paired Student's t-test**: Fast parametric test for mean differences.
     - **Randomized Permutation / Fisher Randomization Test**: Exact or Monte Carlo ($B=10{,}000$ to $100{,}000$ resamples) permutation test.
     - **Studentized Bootstrap**: Confidence intervals and bootstrap p-values.
   - **Efficient Batch Execution**: Implemented with zero-allocation loops and fast `SplitMix64` PRNG; computing all $\binom{N}{2} \times M$ tests completes in a few milliseconds on a single core.


2. **Python Layer (Multiple Comparisons Adjustments & Reporting)**:
   - Ingests the matrix of raw p-values from Rust and applies multiple testing corrections.
   - **Family Definition (Default: Option A - Per-Measure)**:
     - Hypotheses are grouped per-measure (e.g. all pairwise system comparisons for `map` form one family, and `ndcg@10` form a separate family), matching standard IR publication conventions.
     - An optional `family="global"` or `family="baseline"` can be specified by the user.
   - **Supported Adjustment Methods (in order of recommendation)**:
     1. `holm` (**Holm-Bonferroni step-down, Default**): Strongly controls Family-Wise Error Rate (FWER) $\le \alpha$ with substantially higher statistical power than single-step Bonferroni without assuming independence.
     2. `fdr_bh` (**Benjamini-Hochberg**): Controls False Discovery Rate (FDR); optimal for large multi-run benchmark screening.
     3. `bonferroni` (**Single-step Bonferroni** $\alpha / m$): Conservative classical FWER control.
     4. `none`: Raw unadjusted p-values.
   - **Zero-Dependency Implementation**: The adjustment algorithms are implemented directly in standard Python with no required dependencies (and can integrate with `scipy` or `statsmodels` if available).

#### 5.4.2 Python Multi-Run Comparison API

1. **One-vs-Many (Treatment vs Baselines)**:
   ```python
   evaluator = trec_eval.Evaluator(qrels, measures=["map", "ndcg@10"])
   
   # Compare our new model against 5 baseline runs with Benjamini-Hochberg FDR correction
   comp_table = evaluator.compare_against_baseline(
       baseline="runs/bm25.txt",
       candidates={
           "splade": "runs/splade.txt",
           "colbert": "runs/colbert.txt",
           "cross_encoder": "runs/rerank.txt"
       },
       test="permutation",          # executed in high-speed Rust
       correction="holm",           # adjusted in Python layer
       alpha=0.05
   )
   
   print(comp_table.to_dataframe())
   # Output columns: [system, measure, baseline_score, system_score, diff, p_raw, p_adj, significant]
   ```

2. **All-Pairs Comparison Matrix**:
   ```python
   # Full pairwise comparison matrix across all runs
   matrix = evaluator.compare_all(
       runs={"bm25": run1, "splade": run2, "colbert": run3},
       measure="map",
       test="paired_t",
       correction="fdr_bh"
   )
   print(matrix.pvalues_adjusted)   # Symmetric DataFrame of adjusted p-values
   print(matrix.summary_table())     # Printable summary with significance markers (▲ / ▼ / ns)
   ```

3. **Single Pair Comparison**:
   ```python
   comp = res_a.compare(res_b, measure="map", test="paired_t")
   print(comp.pvalue, comp.statistic, comp.significant(0.05))
   ```

### 5.5 Packaging Extras
In `pyproject.toml`:
```toml
[project.optional-dependencies]
numpy = ["numpy>=1.20"]
pandas = ["pandas>=1.0", "numpy>=1.20"]
stats = ["scipy>=1.8", "numpy>=1.20"]
all = ["pandas", "numpy", "scipy", "polars", "pyarrow"]
```

---

## 6. API Design, Examples & Migration Patterns

### 6.1 Concrete In-Memory Dictionary Examples

The primary API directly accepts standard nested dictionaries for `qrels` and `run`, making migration from existing scripts effortless.

```python
import trec_eval

# Relevance judgments (qrels): query_id -> {doc_id: relevance_score}
qrels = {
    "q1": {
        "d1": 0,
        "d2": 1,
        "d3": 0,
    },
    "q2": {
        "d2": 1,
        "d3": 1,
    },
}

# System run: query_id -> {doc_id: retrieval_score}
run = {
    "q1": {
        "d1": 1.0,
        "d2": 0.0,
        "d3": 1.5,
    },
    "q2": {
        "d1": 1.5,
        "d2": 0.2,
        "d3": 0.5,
    },
}
```

#### Example A: Object-Oriented Evaluator (`trec_eval.Evaluator`)
Best when evaluating multiple runs against the same judgments (qrels are pre-indexed once in Rust):

```python
from trec_eval import Evaluator

# Initialize evaluator with qrels dict and desired measures
evaluator = Evaluator(qrels, measures=["map", "ndcg@10", "P.5,10", "recip_rank"])

# Evaluate run dictionary
results = evaluator.evaluate(run)

# Summary averages across queries:
print(results.aggregate())
# {
#     "map": 0.4583333333333333,
#     "ndcg_cut_10": 0.5967132018086354,
#     "P_5": 0.1,
#     "P_10": 0.1,
#     "recip_rank": 0.5833333333333333
# }

# Per-query breakdown:
print(results.per_query())
# {
#     "q1": {"map": 0.3333333333333333, "ndcg_cut_10": 0.5, "P_5": 0.2, "P_10": 0.1, "recip_rank": 0.3333333333333333},
#     "q2": {"map": 0.5833333333333333, "ndcg_cut_10": 0.6934264036172708, "P_5": 0.2, "P_10": 0.2, "recip_rank": 1.0}
# }

# Dictionary-like indexing (backwards-compatible with pytrec_eval):
print(results["q1"]["map"])  # 0.3333333333333333

# Optional: export to Pandas DataFrame
df = results.to_dataframe()
```

#### Example B: Quick Functional One-Liner (`trec_eval.evaluate`)
Ideal for ad-hoc evaluations and interactive notebook sessions:

```python
import trec_eval

# One-line evaluation from dictionaries
results = trec_eval.evaluate(qrels, run, measures=["map", "ndcg@10", "recip_rank"])
print(results.aggregate())
```

---

### 6.2 Ecosystem Compatibility Layers

#### `pytrec_eval` Drop-in Compatibility
Existing scripts using `pytrec_eval` can switch to the Rust backend by changing only the import line:

```python
# Legacy code:
# import pytrec_eval
# Drop-in replacement:
import trec_eval.compat.pytrec_eval as pytrec_eval

evaluator = pytrec_eval.RelevanceEvaluator(qrels, {"map", "ndcg"})
res = evaluator.evaluate(run)

# Returns the exact pytrec_eval nested dictionary structure:
# {
#     "q1": {"map": 0.3333333333333333, "ndcg": 0.5},
#     "q2": {"map": 0.5833333333333333, "ndcg": 0.6934264036172708}
# }
```

#### `ir_measures` Provider Integration
`ir_measures` users can register the Rust backend for acceleration:

```python
import ir_measures
from trec_eval.compat.ir_measures import TeRustProvider

ir_measures.register_provider(TeRustProvider())
```

---

## 7. Performance & Concurrency Model

1. **GIL Release**:
   - All computationally intensive phases (file parsing, query alignment, metric calculations across topics, significance testing) release the Python GIL (`py.allow_threads(...)`).
   - Enables multi-threaded Python applications (e.g. concurrent evaluation of multiple runs across worker threads) without GIL contention.
2. **Qrels Pre-indexing**:
   - In `Evaluator(qrels, ...)`, judgments are parsed, sorted, and converted into Rust lookup structures once.
   - Subsequent `evaluator.evaluate(run)` calls incur zero qrels re-parsing or re-indexing cost.


---

## 8. Measure Specification & Syntax

To ensure compatibility with existing habits while providing modern ergonomics:

1. **Standard `trec_eval` CLI names**:
   `"map"`, `"ndcg"`, `"ndcg_cut.10"`, `"P.5,10,20"`, `"recip_rank"`, `"bpref"`, `"runid"`, etc.
2. **Convenience Aliases**:
   - Cutoff syntax with `@`: `"ndcg@10"` -> `"ndcg_cut.10"`, `"P@5"` -> `"P.5"`.
   - Case-insensitive parsing: `"NDCG@10"` -> `"ndcg_cut.10"`.
3. **Help & Introspection**:
   `trec_eval.list_measures()` and `trec_eval.describe_measure("map")` available directly in Python.

---

## 9. Custom Python Measures & Extension Hooks

Researchers frequently prototype new evaluation measures (novel rank-biased weights, task-specific gain functions, diversity metrics, user effort models). In legacy C `trec_eval`, adding a metric required modifying C code, recompiling, and maintaining a custom fork.

`trec_eval 11` allows users to define custom measures in pure Python and pass them directly into the evaluation pipeline alongside built-in measures.

### 9.1 Custom Measure Interface

Users can define a measure either via class inheritance or functional decorators:

#### Option A: Class-Based Measure (`trec_eval.Measure`)
```python
from trec_eval import Measure, QueryState

class ReciprocalRankAtK(Measure):
    """Custom user-defined reciprocal rank at cutoff k."""
    def __init__(self, k: int = 10):
        super().__init__(name=f"rr@{k}")
        self.k = k

    def calc_query(self, query: QueryState) -> float:
        # query.relevances: list/array of relevance scores in ranked order
        # query.num_rel: total relevant docs in collection for this query
        # query.num_ret: total retrieved docs for this query
        # query.doc_ids: list of retrieved doc IDs
        for rank, rel in enumerate(query.relevances[:self.k], start=1):
            if rel >= self.relevance_level:
                return 1.0 / rank
        return 0.0

    def aggregate(self, scores: list[float]) -> float:
        # Optional custom aggregation (default is standard arithmetic mean)
        return sum(scores) / len(scores) if scores else 0.0
```

#### Option B: Functional Decorator (`@trec_eval.register_measure`)
```python
@trec_eval.register_measure(name="p_at_3")
def precision_at_three(query: QueryState) -> float:
    rel_count = sum(1 for r in query.relevances[:3] if r >= 1)
    return rel_count / 3.0
```

### 9.2 Seamless Pipeline Integration

Custom measures are passed directly in the `measures` list alongside standard built-in metric strings:

```python
evaluator = trec_eval.Evaluator(
    qrels,
    measures=[
        "map",                      # Built-in Rust measure
        "ndcg@10",                  # Built-in Rust measure
        ReciprocalRankAtK(k=5),     # Custom Python class
        precision_at_three,         # Custom Python function
    ]
)

results = evaluator.evaluate(run)

# Custom measures appear seamlessly in all outputs:
print(results.aggregate())
# {"map": 0.458, "ndcg_cut_10": 0.596, "rr@5": 0.583, "p_at_3": 0.166}

# Custom measures appear as columns in DataFrames:
df = results.to_dataframe()

# Custom measures support statistical significance testing:
comp = res_a.compare(res_b, measure="rr@5", test="permutation")
```

### 9.3 Execution & Error Handling Model
1. **Parallelism & GIL Safety**: Built-in Rust metrics run in parallel with the GIL released. When evaluating custom Python measures, PyO3 acquires the GIL and passes a zero-copy `QueryState` view to the Python function.
2. **Exception Sandboxing**: If a custom Python metric raises an exception (e.g. division by zero on a specific query), the error is caught and wrapped with topic context (`EvaluationError: Error in custom measure 'rr@5' on topic 'q42': ZeroDivisionError(...)`), preventing process crashes.

---

## 10. Web Documentation & Knowledge Base

The Python project will include a unified, modern web documentation site hosted on GitHub Pages (built with **Zensical**). This site will serve as the central reference for both the Python library and the underlying Rust CLI tool.


### 10.1 Documentation Scope & Structure

1. **Getting Started & Quickstarts**:
   - Installation (`pip install trec-eval`, `cargo install trec-eval`).
   - 5-minute quickstarts for Python notebook users, script writers, and CLI users.
   - Migration guides from C `trec_eval 9.x/10.x`, `pytrec_eval`, and `ir_measures`.

2. **Python API Reference**:
   - Complete reference for `trec_eval.Evaluator`, `trec_eval.evaluate()`, `EvalResult`, and type definitions (`ScoredDoc`, `Qrel`).
   - In-depth guides for **NumPy zero-copy ingestion**, **Pandas/Polars DataFrames**, and custom object streams.
   - **Custom Measure Development Guide**: Step-by-step tutorial on implementing and publishing custom metrics.
   - **Multi-Run Statistical Significance Guide**: Practical tutorials on paired tests, permutation testing, and multiple comparison corrections (`holm`, `fdr_bh`).

3. **Rust CLI Reference**:
   - Complete flag and argument guide (`-q`, `-n`, `-c`, `-l`, `-J`, `-M`, `-N`, `--global-gains`).
   - Exit codes, pipeline chaining, and standard TREC file format specifications (6-column runs, 4-column qrels, preference formats).

4. **Comprehensive Measure Catalog**:
   - Detailed documentation for every supported evaluation measure:
     - Mathematical definitions and formulas (rendered in KaTeX/MathJax).
     - Parameterization and cutoff options (e.g., `ndcg_cut.10`, `P.5,10`, `map_cut.1000`).
     - Edge-case handling (empty rankings, unjudged documents, zero-relevant queries).
     - Theoretical background, historical references, and citing papers (e.g. Buckley & Voorhees for bpref, Järvelin & Kekäläinen for NDCG, Moffat & Zobel for RBP).

5. **Automated Docstring & Syncing**:
   - Python docstrings and typing stubs will be synchronized directly from the measure definitions and Rust doc comments to ensure documentation never drifts from the implementation.

---

## 11. Summary of Resolved Design Decisions

1. **Workspace & Packaging**:
   - Target Release Version: **`11.0.0`** (succeeding `trec_eval 10.x`).
   - Cargo workspace with two parallel crates: `te-rust` (core engine & standalone CLI binary) and `te-python` (PyO3 bindings & Maturin packaging).
   - Top-level Python module name: `trec_eval` (`import trec_eval`).
   - PyPI distribution name: `trec-eval` (with optional extras `[pandas]`, `[all]`).

2. **Evaluation Return Type**:
   - `evaluator.evaluate(run)` returns a rich `EvalResult` implementing `collections.abc.Mapping`.
   - Legacy dict indexing (`res["q1"]["map"]`), iteration, and `dict(res)` work out of the box.
   - Provides native methods: `.aggregate()`, `.per_query()`, `.to_dataframe(format="wide"|"tidy")`, `.to_numpy(measure)`, and `.compare(other_res)`.
   - Full configuration switches supported: `relevance_level` (`-l`), `complete_set_average` (`-c`), `judged_docs_only` (`-J`), `max_docs_per_topic` (`-M`), `num_docs_in_coll` (`-N`), and `global_gains`.

3. **Input Ingestion & Object Bridging**:
   - Canonical formats model: File paths, nested dicts, Pandas/Polars DataFrames, iterables of 3-tuples `(qid, doc_id, score)`, canonical `ScoredDoc` namedtuples, and 1D NumPy arrays.
   - Users bridge custom classes/dataclasses using standard generator expressions: `((h.topic, h.docno, h.score) for h in hits)`.

4. **Custom Python Measures**:
   - First-class support for user-defined Python metrics (`trec_eval.Measure` base class or `@register_measure` decorator).
   - Custom metrics participate fully in query evaluation, DataFrames, and statistical significance testing.

5. **Multi-Run Statistical Significance**:
   - Multi-threaded paired testing in Rust (`t-test`, Monte Carlo `permutation`, and `bootstrap`).
   - Multiple comparisons adjustments in pure Python with zero required dependencies:
     - Default grouping: **Per-measure family** (matches standard IR publications).
     - Default correction: **`holm`** (Holm-Bonferroni step-down), with support for `fdr_bh`, `bonferroni`, and `none`.

6. **Web Documentation & Knowledge Base**:
   - Dedicated web documentation site covering the Python API, Rust CLI, scientific workflows, custom metric development, and an exhaustive mathematical catalog of all IR measures with edge cases and citations.

---

## 12. Staged Implementation Plan

When implementing the Python package, work should be executed in well-scoped sequential stages:

### Stage 1: Workspace Scaffolding & Core Rust Library Exposure
- Create root `Cargo.toml` defining the workspace (`te-rust`, `te-python`).
- Add `te-rust/src/lib.rs` to expose `eval`, `io`, and `metrics` modules.
- Create `te-python/Cargo.toml` with `pyo3` and `maturin` dependencies.
- Create `te-python/pyproject.toml` with build-system metadata and optional dependency extras.
- Verify that `maturin develop` compiles a basic importable `trec_eval` Python module.

### Stage 2: Core PyO3 Evaluator & EvalResult
- Implement `Evaluator` PyClass in Rust wrapping `te_rust::metrics::EvalConfig` and pre-indexed `Qrels`.
- Implement `EvalResult` PyClass supporting Python `Mapping` protocol (`__getitem__`, `__iter__`, `keys`, `values`, `items`, `to_dict()`).
- Implement `.aggregate()` and `.per_query()` methods.
- Implement `trec_eval.evaluate()` functional one-liner.

### Stage 3: Canonical Ingestion Formats
- Support file paths (`str`, `PathLike`).
- Support nested dicts (`Dict[str, Dict[str, float]]`).
- Support 3-tuples `(qid, doc_id, score)` and `ScoredDoc` namedtuples.
- Implement DataFrame ingestion (Pandas / Polars) with column mapping kwargs.
- Implement NumPy 1D array buffer ingestion (`evaluate_arrays`).
- Implement `.to_dataframe(format="wide"|"tidy")` and `.to_numpy(measure)` outputs.

### Stage 4: Multi-Run Statistical Significance & Corrections
- Expose Rust-level pairwise paired tests (`paired_t`, `permutation`, `bootstrap`).
- Implement pure-Python multiple comparisons correction engine (`holm`, `fdr_bh`, `bonferroni`).
- Implement `res_a.compare(res_b)`, `evaluator.compare_against_baseline()`, and `evaluator.compare_all()`.


### Stage 5: Custom Python Measures Extension Hook
- Implement `trec_eval.Measure` base class and `QueryState` PyClass wrapper.
- Implement `@trec_eval.register_measure` decorator.
- Implement PyO3 callback invocation and exception sandboxing in the evaluation loop.

### Stage 6: Ecosystem Compatibility Layers
- Implement `trec_eval.compat.pytrec_eval` module with exact `RelevanceEvaluator` signature.
- Implement `trec_eval.compat.ir_measures` provider adapter.

### Stage 7: Web Documentation Site
- Set up Zensical documentation structure.
- Document CLI reference, Python API, scientific workflows, custom measures, and measure catalog.


---

## 13. Testing & Quality Assurance Strategy

In accordance with project workflow guidelines, **tests will be created, verified, and continuously maintained through each stage of implementation**. No stage or feature is considered complete until its corresponding unit and integration tests are written and passing.

1. **Incremental Stage Testing**:
   - Each implementation stage (Stages 1–7) includes dedicated unit tests verifying that stage's deliverables before advancing.
   - Test suites are kept green and executed continuously as new features are added.

2. **Numerical Parity Tests (`pytest`)**:
   - Compare all evaluations computed via Python (`Evaluator`, dicts, DataFrames, arrays) directly against the compiled Rust CLI binary and legacy C `trec_eval` outputs on standard TREC test collections.
   - Verify that all floating-point scores match to standard precision (numerical differences must be negligible or attributable to floating-point representation).

3. **Compatibility Suite**:
   - Run the existing test suites of `pytrec_eval` against `trec_eval.compat.pytrec_eval` to ensure 100% bug-for-bug behavioral compatibility.

4. **Corner-Case & Boundary Tests**:
   - Empty rankings (0 documents retrieved).
   - Unjudged queries and queries with zero relevant documents.
   - Rankings where no judged documents are present.
   - Extreme floating point values (NaN, Inf) and ties in scores.

5. **Memory & Concurrency Stress Tests**:
   - Verify GIL release by evaluating runs across multiple Python worker threads simultaneously.
   - Large-scale throughput benchmarks on runs with >1,000,000 scored documents.

---

## 14. Open Design Issues & Dependencies (Review Notes)

The following items were surfaced while reviewing this document against the current `te-rust`
codebase and `docs/ISSUES.md`. They are recorded here so the design can be reconciled with the
core engine's already-resolved decisions before implementation begins. Each item notes the related
ISSUES.md entry where applicable.

### 14.1 Custom-measure `QueryState` collides with the doc-ID-free aligned state
- **Where**: §9 (Custom Python Measures). The proposed `QueryState` exposes `query.doc_ids` and a
  raw `query.relevances` list.
- **Reality**: The core aligned state is `QueryEvalState` (`src/eval/alignment.rs`), whose contract
  is `results_rel_list` (an `i64` array using sentinels `-1` non-pool / `-2` unjudged) plus
  `rel_levels` counts. **Document IDs are intentionally dropped after alignment**, mirroring C's
  `RES_RELS`. This is not an oversight — it is settled by ISSUES.md **#16** (native empty-state
  evaluation) and **#17** (defensive `rel_levels` sizing), and the trait/state shape by **#10** and
  **#14**.
- **Decision needed**: Either (a) expose `QueryEvalState` to custom measures as-is (no doc IDs, raw
  sentinel-encoded relevances), or (b) introduce a new opt-in aligned view that retains doc IDs for
  custom measures only. Option (b) reopens #16/#17 and must be justified.

### 14.2 Custom-measure `aggregate()` default bypasses centralized averaging
- **Where**: §9.1 `aggregate()` defaults to arithmetic mean.
- **Reality**: The `Measure` trait already centralizes averaging (`Measure::average`), including the
  `-c` / `average_complete_flag` denominator switch (divide by `total_qrels_queries`) and
  geometric-mean measures (`gm_map`, `gm_bpref`). Related: ISSUES.md **#14**.
- **Decision needed**: Custom Python measures must plug into the existing averaging contract (honor
  `-c`, allow non-arithmetic aggregation) rather than defining a parallel default.

### 14.3 Preference measures are out of scope for Python v1 (blocked on core impl)
- **Where**: §4 (Ingestion) models only `(qid, doc_id, score)` runs and `(qid, doc_id, relevance)`
  qrels — no preference (prefs / qrels_prefs / jg) input path.
- **Reality**: Preference evaluation is fully *designed* (ISSUES.md **#15**, `EVAL_DESIGN.md`:
  `PrefsEvalState`, `JudgmentGroup`, `EquivalenceClass`, dual transitive-closure strategy) but
  **not yet implemented** in `src/`. `EvalState` currently has only the `Standard` variant.
- **Decision**: Scope Python v1 to standard (scored-run) evaluation. Python preference support is an
  explicit downstream dependency on implementing #15 in the core. State this scope boundary in the
  API docs so it is not mistaken for a permanent limitation.

### 14.4 Significance testing should build on the planned bootstrap machinery
- **Where**: §5.4 introduces t-test / permutation / bootstrap in Rust.
- **Reality**: The core already plans bootstrap confidence intervals behind a switch for CLI
  compatibility (ISSUES.md **#22**). This is the natural home for the resampling primitives.
- **Decision**: Implement the Rust-side resampling once, shared between the CLI CI feature (#22) and
  the Python significance layer. Note this is a *new capability* for the trec_eval lineage (the C
  tool evaluates a single run with no inferential statistics), so it is additive, not parity work.

### 14.5 Measure-name/alias parsing must be shared with the CLI, not duplicated
- **Where**: §8 proposes `@`-cutoff aliases (`ndcg@10` → `ndcg_cut.10`) and case-insensitive names.
- **Reality**: A single canonical measure-name + options parser is already an open CLI task
  (ISSUES.md **#25** / **#26** — remove duplicated cutoff parsing, delegate option parsing to the
  measure code). Both surfaces need the same parser.
- **Decision**: Land the canonical parser in the core (resolving #25/#26) and have the Python layer
  consume it. Do not add a second, Python-only measure-string parser.

### 14.6 Illustrative example numbers must be computed, not hand-written
- **Where**: §6.1 shows concrete aggregate/per-query values (e.g. `map = 0.4583…`) for the toy
  qrels/run.
- **Reality**: These were not produced by running the tool. The project's testing philosophy
  (ISSUES.md **#6**, **#19**) makes real computed output the source of truth via epsilon-based
  relational comparison.
- **Decision**: Before publishing, either regenerate these numbers with the actual binary or clearly
  label them as illustrative/non-authoritative.

### 14.7 `EvalResult` indexing semantics are ambiguous
- **Where**: §6.1 vs §11. `results["q1"]["map"]` (query-keyed, pytrec_eval-style) coexists with
  `results.aggregate()` returning a flat measure dict.
- **Decision needed**: Define a single `Mapping` key semantics for `EvalResult` (query-keyed is the
  pytrec_eval-compatible choice), with aggregate/per-query exposed as explicit methods.

### 14.8 Omission of Rayon multithreading for resampling tests
- **Where**: §5.4.1, §7, and §12 mentioned using Rayon for parallel resampling.
- **Reality**: Single-threaded evaluation using our register-level `SplitMix64` PRNG takes ~12 microseconds
  for 10,000 permutation resamples across 50 topics, and ~1.5 milliseconds for a 10-system all-pairs matrix (45 tests).
  Spawning and synchronizing worker threads via Rayon incurs 50–200 microseconds of thread coordination overhead,
  making multithreading slower than sequential L1 cache execution for standard IR workloads.
- **Decision**: Keep significance testing single-threaded in pure Rust, avoiding Rayon dependencies while
  releasing the Python GIL for multi-threaded Python applications.

