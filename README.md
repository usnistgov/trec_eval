# trec-eval-rust

A compatible reimplementation of `trec_eval` in Rust, providing identical usage and exact numerical parity with the standard Information Retrieval evaluation tool, a standalone command-line interface, and native high-performance Python bindings.

## Documentation

Documentation (built from the docs/ directory) is available at https://isoboroff.github.io/trec-eval-rust/.

## Project Structure

```text
trec-eval-rust/
├── Cargo.toml                # Top-level workspace configuration
├── te-rust/                  # Core Rust library & CLI executable
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs            # Core library (parsers, alignment, metrics, eval engine)
│   │   ├── main.rs           # Standalone CLI binary (`te-rust`)
│   │   ├── eval/             # Alignment and evaluation execution engine
│   │   ├── io/               # Standard TREC format parsers (qrels, runs)
│   │   └── metrics/          # Stateless evaluation measures and registry
│   ├── docs/                 # Design documents and issue resolutions
│   └── tests/                # Verification and live C-parity regression suites
├── te-python/                # Python bindings package (`trec-eval`)
│   ├── Cargo.toml            # PyO3 extension crate (`_trec_eval`)
│   ├── pyproject.toml        # PEP 517/621 packaging metadata and extras
│   ├── python/               # Pure Python package layer (`trec_eval`)
│   ├── src/                  # Rust PyO3 classes (Evaluator, EvalResult, conversions)
│   └── tests/                # Python integration & numerical parity tests
└── trec_eval/                # Reference C trec_eval checkout (read-only reference)
```

---

## Building and Testing

### Prerequisites

- **Rust**: 1.75+ (via `rustup`)
- **Python**: 3.8+
- **uv** (recommended) or `pip` + `maturin`

### 1. Rust CLI (`te-rust`)

```bash
# Build optimized CLI binary
cargo build --release

# Run CLI directly
cargo run --release -- -q -m official qrels.test results.test

# Run all Rust unit tests, boundary invariant tests, and C regression tests
cargo test
```

The compiled binary will be located at `target/release/te-rust`.

### 2. Python Package (`trec-eval`)

```bash
# Install development build into current Python environment
cd te-python
uv run maturin develop
cd ..

# Run Python test suite with all dev dependencies (Pandas, NumPy, SciPy):
uv run --project te-python --extra dev pytest te-python/tests

# Or run in a minimal zero-dependency environment:
uv run --with pytest pytest te-python/tests
```


---

## Quickstart & Usage

### Command-Line Interface (`te-rust`)

Compatible with standard `trec_eval` flags and syntax:

```bash
# Evaluate standard official measures
te-rust qrels.txt run.txt

# Print query-level breakdown and specific measures with cutoffs
te-rust -q -m map -m P.5,10,20 -m ndcg@10 -m recip_rank qrels.txt run.txt

# Compute 95% bootstrap confidence intervals for summary averages
te-rust -C --ci-pretty -m official qrels.txt run.txt

# Inspect available measures and parameter options
te-rust --help-measures
te-rust --help-measure ndcg_cut
```

### Python API (`trec_eval`)

The Python package provides zero-dependency evaluation supporting nested dictionaries, file paths, and row iterables:

```python
import trec_eval

# Relevance judgments: {query_id: {doc_id: relevance}}
qrels = {
    "q1": {"d1": 0, "d2": 1, "d3": 0},
    "q2": {"d2": 1, "d3": 1},
}

# System run: {query_id: {doc_id: score}}
run = {
    "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
    "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
}

# Quick functional evaluation
results = trec_eval.evaluate(qrels, run, measures=["map", "ndcg@10", "recip_rank"])

# Summary averages across topics:
print(results.aggregate())
# {'map': 0.4583, 'ndcg_cut_10': 0.5967, 'recip_rank': 0.5833}

# Per-query breakdown:
print(results.per_query())

# Dictionary-like query indexing (pytrec_eval compatibility):
print(results["q1"]["map"])  # 0.3333
```

#### Object-Oriented `Evaluator`

When evaluating multiple runs against the same judgments, pre-index qrels once:

```python
from trec_eval import Evaluator

# Pre-indexes judgments and configuration in Rust
evaluator = Evaluator(
    qrels="path/to/qrels.txt",
    measures=["map", "ndcg@10", "P.5,10", "bpref"],
    relevance_level=1,
    complete_set_average=False,
)

# Evaluate runs with GIL released for multi-threaded performance
res1 = evaluator.evaluate("path/to/run1.txt")
res2 = evaluator.evaluate("path/to/run2.txt")
```

## Statement on AI-assisted design

This project was started as perhaps my first major AI-assisted coding project. Both the design and the code were developed with the assistance of LLMs. The process was fundamentally interactive and adversarial -- I would prompt the model to propose designs, plans, or code, and those outputs were successively worked through multiple back-and-forth rounds, sometimes with different models giving opposing reviews. A helpful technique I found was to start an LLM interaction with a clean slate and ask a model to do a full code-review (or design review) without implying that anything was written with AI. All code was manually reviewed during development before inclusion, and I take full responsibility for the project.
