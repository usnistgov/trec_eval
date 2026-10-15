# Evaluator & EvalResult API Reference

## `trec_eval.Evaluator`

```python
class Evaluator(
    qrels: Any,
    measures: Optional[Union[str, List[str], Set[str]]] = None,
    relevance_level: int = 1,
    complete_set_average: bool = False,
    judged_docs_only: bool = False,
    max_docs_per_topic: Optional[int] = None,
    num_docs_in_coll: int = 0,
    global_gains: Optional[str] = None,
    qid: Optional[str] = None,
    docno: Optional[str] = None,
    rel: Optional[str] = None,
)
```

### Parameters

- `qrels` (`str`, `PathLike`, `dict`, `DataFrame`, or `Iterable`): Relevance judgments.
- `measures` (`str`, `list[str]`, or `list[Measure]`, default `'official'`): Measure names, cutoff aliases, groups (`'official'`, `'all_trec'`, `'set'`), or custom `Measure` instances.
- `relevance_level` (`int`, default `1`): Minimum grade at which a document is considered relevant (`-l`).
- `complete_set_average` (`bool`, default `False`): When `True`, averages over all queries in `qrels` rather than only queries present in the run (`-c`).
- `judged_docs_only` (`bool`, default `False`): When `True`, drops all unjudged documents from the ranking before evaluation (`-J`).
- `max_docs_per_topic` (`int`, optional): Truncates the ranking to at most this many documents per topic (`-M`).
- `num_docs_in_coll` (`int`, default `0`): Total collection size (`-N`).
- `global_gains` (`str`, optional): Global gain mapping string (e.g. `'1=3.5,2=9.0'`).
- `qid`, `docno`, `rel` (`str`, optional): Explicit column names when `qrels` is a DataFrame.

---

### Methods

#### `evaluate`

```python
evaluate(run, qid=None, docno=None, score=None) -> EvalResult
```

Evaluates a system run against the pre-indexed judgments. The Python GIL is released during computation.

- `run` (`str`, `PathLike`, `dict`, `DataFrame`, or `Iterable`): System run results.
- `qid`, `docno`, `score` (`str`, optional): Column name overrides for DataFrame inputs.

#### `evaluate_arrays`

```python
evaluate_arrays(query_ids, doc_ids, scores) -> EvalResult
```

Evaluates 1D arrays or contiguous buffer slices directly without creating intermediate Python objects.

---

## `trec_eval.EvalResult`

The result object returned by `evaluator.evaluate()` and `trec_eval.evaluate()`. It implements `collections.abc.Mapping`, supporting query-level indexing and dict conversions matching `pytrec_eval`.

### Inspection & Output Methods

#### `.aggregate()`

```python
aggregate() -> Dict[str, float]
```

Returns the summary aggregate score across all evaluated queries for each requested measure.

```python
agg = results.aggregate()
print(agg["map"])        # 0.2543
print(agg["ndcg_cut_10"]) # 0.3812
```

#### `.per_query()`

```python
per_query() -> Dict[str, Dict[str, float]]
```

Returns the complete nested dictionary of per-topic scores: `Dict[qid, Dict[measure_name, score]]`.

```python
queries = results.per_query()
print(queries["301"]["map"]) # 0.3120
```

#### `.to_dict()`

```python
to_dict() -> Dict[str, Dict[str, float]]
```

Alias for `.per_query()`, providing full dictionary compatibility.

#### `.to_dataframe()`

```python
to_dataframe(format="wide") -> pandas.DataFrame
```

Exports the evaluation scores as a Pandas or Polars DataFrame.

- `format="wide"` (default): Query IDs as rows, measures as columns.
- `format="tidy"` (or `'long'`): 3-column long format `(query_id, measure, value)`.

*Requirement*: Requires `pandas` (`pip install "trec-eval[pandas]"`) or `polars`. Raises `ImportError` if neither is installed.

```python
df_wide = results.to_dataframe(format="wide")
#    query_id     map  ndcg_cut_10  recip_rank
# 0       301  0.3120       0.4500      0.5000
# 1       302  0.1980       0.2800      0.2500

df_tidy = results.to_dataframe(format="tidy")
#    query_id      measure   value
# 0       301          map  0.3120
# 1       301  ndcg_cut_10  0.4500
```

#### `.to_numpy()`

```python
to_numpy(measure="map") -> numpy.ndarray
```

Returns a 1D NumPy array of per-query scores for the specified measure in deterministic topic ordering (matching `.keys()`).

*Fallback*: If `numpy` is installed (`pip install "trec-eval[numpy]"`), returns a `numpy.ndarray`. If NumPy is not installed, gracefully returns a standard Python `list[float]`.

```python
map_scores = results.to_numpy("map")
print(map_scores.mean(), map_scores.std())
```


#### `.compare()`

```python
compare(other, measure="map", test="paired_t", num_resamples=10000, seed=None) -> ComparisonResult
```

Performs a statistical significance test between this run and another `EvalResult` across aligned topic scores.

```python
comp = res_a.compare(res_b, measure="map", test="paired_t")
print(comp.statistic, comp.pvalue, comp.significant(0.05))
```


---

### Mapping Protocol Support

| Operation | Description |
| :--- | :--- |
| `results[qid]` | Returns the metric score dictionary for query `qid`. Raises `KeyError` if missing. |
| `qid in results` | Checks if `qid` is in the evaluated topics. |
| `len(results)` | Number of evaluated queries. |
| `list(results)` / `iter(results)` | Iterates over evaluated query IDs. |
| `results.keys()` | Returns a list of evaluated query IDs. |
| `results.values()` | Returns a list of per-query score dictionaries. |
| `results.items()` | Returns `(qid, score_dict)` tuples. |
| `dict(results)` | Converts `EvalResult` to a standard Python dictionary. |
