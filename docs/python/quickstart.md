# Python Quickstart & Ingestion Formats

`trec-eval` accepts data from files, in-memory nested dictionaries, row iterables, Pandas/Polars DataFrames, and NumPy 1D arrays.

---

## 1. Functional One-Liner (`trec_eval.evaluate`)

For quick ad-hoc evaluations in scripts or interactive notebooks, use `trec_eval.evaluate()`:

```python
import trec_eval

# Relevance judgments: Dict[query_id, Dict[doc_id, relevance_grade]]
qrels = {
    "q1": {"d1": 0, "d2": 1, "d3": 0},
    "q2": {"d2": 1, "d3": 1},
}

# System run results: Dict[query_id, Dict[doc_id, score]]
run = {
    "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
    "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
}

# Evaluate
results = trec_eval.evaluate(qrels, run, measures=["map", "ndcg@10", "recip_rank"])

# Summary averages across queries
print(results.aggregate())
# {'map': 0.4583, 'ndcg_cut_10': 0.5967, 'recip_rank': 0.5833}

# Per-query evaluations
print(results["q1"]["map"])  # 0.3333
```

---

## 2. Pre-Indexed `Evaluator`

When evaluating multiple runs against the same relevance judgments, construct an `Evaluator` object. The judgments and measure specifications are parsed and indexed once in Rust, and subsequent calls to `.evaluate(run)` run without re-indexing.

```python
from trec_eval import Evaluator

evaluator = Evaluator(
    qrels="qrels.txt",
    measures=["map", "ndcg@10", "P.5,10", "recip_rank"],
    relevance_level=1,
    complete_set_average=False,
)

res_bm25 = evaluator.evaluate("runs/bm25.txt")
res_splade = evaluator.evaluate("runs/splade.txt")
```

---

## 3. Supported Input Formats

### A. File Paths (`str` or `os.PathLike`)

Paths to standard 4-column qrels and 6-column run files are read directly in Rust with buffered I/O, bypassing Python object allocation:

```python
evaluator = Evaluator("data/qrels.txt", measures=["official"])
results = evaluator.evaluate("data/run.txt")
```

### B. Nested Dictionaries

Compatible with `pytrec_eval`:

```python
qrels = {
    "topic_1": {"doc_A": 1, "doc_B": 0},
    "topic_2": {"doc_C": 2, "doc_D": 1},
}
run = {
    "topic_1": {"doc_A": 14.2, "doc_B": 8.1},
    "topic_2": {"doc_C": 12.0, "doc_D": 3.4},
}
results = trec_eval.evaluate(qrels, run)
```

### C. Iterables of Tuples or `NamedTuple`s

Streams of 3-tuples `(query_id, doc_id, score_or_relevance)` or canonical `ScoredDoc` / `Qrel` namedtuples:

```python
from trec_eval import ScoredDoc, Qrel, evaluate

qrels = [
    Qrel("q1", "d1", 0),
    Qrel("q1", "d2", 1),
    Qrel("q2", "d2", 1),
]

run = [
    ScoredDoc("q1", "d1", 0.5),
    ScoredDoc("q1", "d2", 1.5),
    ScoredDoc("q2", "d2", 0.8),
]

results = evaluate(qrels, run, measures=["map", "recip_rank"])
```

#### Bridging Custom Objects and Dataclasses

Custom search hit objects or dataclasses can be passed using generator expressions:

```python
results = evaluator.evaluate(
    (hit.topic, hit.docno, hit.similarity) for hit in search_results
)
```

### D. Pandas and Polars DataFrames

DataFrames with standard column names (`query_id`/`qid`, `doc_id`/`docno`, `score`/`similarity`, `relevance`/`rel`) are ingested directly without converting to intermediate dictionaries.

!!! note "DataFrame Ingestion vs. Export"
    - **Ingestion**: Ingesting DataFrames into `Evaluator` or `evaluate()` uses generic column access and works with any DataFrame object.
    - **Exporting (`.to_dataframe()`)**: Generating a `pandas.DataFrame` or `polars.DataFrame` from results requires `pandas` (`pip install "trec-eval[pandas]"`) or `polars`.

```python
import pandas as pd
import trec_eval


qrels_df = pd.DataFrame({
    "query_id": ["q1", "q1", "q2"],
    "doc_id": ["d1", "d2", "d2"],
    "relevance": [0, 1, 1],
})

run_df = pd.DataFrame({
    "query_id": ["q1", "q1", "q2"],
    "doc_id": ["d1", "d2", "d2"],
    "score": [0.5, 1.5, 0.8],
})

results = trec_eval.evaluate(qrels_df, run_df, measures=["map", "ndcg@10"])
```

#### Custom DataFrame Column Names

If your DataFrame columns use custom names, pass them via column mapping arguments:

```python
results = trec_eval.evaluate(
    qrels_df,
    run_df,
    measures=["map"],
    qid="topic_id",
    docno="document_number",
    rel="relevance_grade",
    score="model_score",
)
```

### E. 1D NumPy Arrays (`evaluate_arrays`)

When retrieval scores are stored in contiguous 1D arrays, evaluate them directly via `evaluate_arrays`:

```python
import numpy as np

query_ids = ["q1", "q1", "q2", "q2"]
doc_ids = ["d1", "d2", "d2", "d3"]
scores = np.array([0.1, 0.9, 0.8, 0.2], dtype=np.float64)

results = evaluator.evaluate_arrays(query_ids, doc_ids, scores)
```
