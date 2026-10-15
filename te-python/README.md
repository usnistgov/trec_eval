# trec-eval Python Package

High-performance Python bindings for `trec_eval`, powered by `te-rust`.

## Installation

```bash
pip install trec-eval
```

## Dependencies & Optional Features

`trec-eval` has **zero mandatory runtime dependencies**. All core evaluation, hypothesis testing (`paired_t`, `permutation`, `bootstrap`), multiple comparison corrections (`holm`, `fdr_bh`, `bonferroni`), and custom measure authoring work with standard Python.

Optional extras are available for tabular and scientific array outputs:

- `pip install "trec-eval[pandas]"`: Enables exporting results to Pandas/Polars DataFrames via `.to_dataframe()`. *(Note: DataFrame ingestion into `Evaluator` works with or without this extra).*
- `pip install "trec-eval[numpy]"`: Returns a `numpy.ndarray` from `.to_numpy(measure)`. *(Falls back to a standard Python `list[float]` if NumPy is not installed).*
- `pip install "trec-eval[all]"`: Installs all optional packages (Pandas, Polars, NumPy).



