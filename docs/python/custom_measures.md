# Custom Evaluation Measures

`trec-eval` allows users to define custom evaluation measures in Python. Custom measures integrate with built-in measures, participating in query evaluation, DataFrames, and significance testing.

---

## 1. Defining Custom Measures

### Option A: Class-Based (`trec_eval.Measure`)

Subclass `trec_eval.Measure` and implement `calc_query(query: QueryState) -> float`:

```python
from trec_eval import Evaluator, Measure, QueryState

class ReciprocalRankAtK(Measure):
    """Reciprocal rank evaluated at rank cutoff k."""

    def __init__(self, k: int = 10):
        super().__init__(name=f"rr@{k}")
        self.k = k

    def calc_query(self, query: QueryState) -> float:
        # Search for first relevant document up to rank k
        for rank, rel in enumerate(query.relevances[: self.k], start=1):
            if rel >= query.relevance_level:
                return 1.0 / rank
        return 0.0

    def aggregate(
        self,
        scores: list[float],
        num_queries_evaluated: int,
        total_qrels_queries: int,
        complete_set_average: bool,
    ) -> float:
        """Custom summary aggregation (default: arithmetic mean)."""
        denominator = total_qrels_queries if complete_set_average else num_queries_evaluated
        return sum(scores) / denominator if denominator > 0 else 0.0
```

### Option B: Functional Decorator (`@trec_eval.register_measure`)

Decorate a Python function that accepts a `QueryState`:

```python
from trec_eval import QueryState, register_measure

@register_measure(name="precision_at_3")
def p_at_three(query: QueryState) -> float:
    rel_count = sum(1 for r in query.relevances[:3] if r >= query.relevance_level)
    return rel_count / 3.0
```

---

## 2. The `QueryState` Object

The `QueryState` object is passed to `calc_query()` for each query topic:

| Attribute | Type | Description |
| :--- | :--- | :--- |
| `query.qid` | `str` | Topic ID. |
| `query.relevances` | `List[int]` | Aligned list of relevance judgments for retrieved documents in ranked order (index $0$ is rank 1). |
| `query.num_rel` | `int` | Total relevant documents in qrels for this query ($\text{judgment} \ge \text{relevance\_level}$). |
| `query.num_ret` | `int` | Total retrieved documents evaluated for this topic. |
| `query.num_rel_ret` | `int` | Total retrieved documents that are relevant. |
| `query.num_nonpool` | `int` | Total retrieved documents not present in the judgment pool (`-1`). |
| `query.num_unjudged_in_pool` | `int` | Total retrieved documents marked unjudged in pool (`-2`). |
| `query.rel_levels` | `List[int]` | Total documents in qrels at each relevance grade (`rel_levels[grade]`). |
| `query.relevance_level` | `int` | Minimum relevance threshold cutoff configured for this evaluation (`-l`). |

### Relevance Value Sentinels in `query.relevances`

- $\ge 0$: Document was judged with that relevance grade ($0 = \text{non-relevant}$, $1, 2, \dots = \text{graded relevance}$).
- `-1`: Document was retrieved but was not judged (not present in qrels).
- `-2`: Document was in pool but marked unjudged (e.g. for infAP sampling).

---

## 3. Using Custom Measures in Evaluation

Pass instances or decorated functions in the `measures` list:

```python
evaluator = Evaluator(
    qrels="qrels.txt",
    measures=[
        "map",                  # Built-in Rust measure
        "ndcg@10",              # Built-in Rust measure
        ReciprocalRankAtK(k=5), # Custom class
        p_at_three,             # Custom function
    ],
)

results = evaluator.evaluate("run.txt")

print(results.aggregate())
# {'map': 0.2543, 'ndcg_cut_10': 0.3812, 'rr@5': 0.6124, 'precision_at_3': 0.4560}
```

---

## 4. Error Handling and Exception Sandboxing

If a custom Python measure raises an exception during evaluation (such as a `ZeroDivisionError` or invalid array index), `trec_eval` catches the exception and wraps it with the specific measure and query ID context:

```text
ValueError: EvaluationError in custom measure 'my_metric' on query '301': ZeroDivisionError: division by zero
```
