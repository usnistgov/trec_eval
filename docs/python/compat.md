# Ecosystem Compatibility Layers

`trec-eval` includes drop-in compatibility modules for existing Python IR libraries.

---

## 1. `pytrec_eval` Compatibility

Scripts written for `pytrec_eval` can switch to `trec-eval` by updating the import:

```python
# Replace: import pytrec_eval
import trec_eval.compat.pytrec_eval as pytrec_eval

qrel = {
    'q1': {'d1': 0, 'd2': 1, 'd3': 0},
    'q2': {'d2': 1, 'd3': 1},
}

run = {
    'q1': {'d1': 1.0, 'd2': 0.0, 'd3': 1.5},
    'q2': {'d1': 1.5, 'd2': 0.2, 'd3': 0.5},
}

# Construct RelevanceEvaluator with measure set
evaluator = pytrec_eval.RelevanceEvaluator(qrel, {'map', 'ndcg'})

# Returns exact nested dictionary format matching pytrec_eval
results = evaluator.evaluate(run)
# {
#     'q1': {'map': 0.3333333333333333, 'ndcg': 0.5},
#     'q2': {'map': 0.5833333333333333, 'ndcg': 0.6934264036172708}
# }
```

### Parsing Utilities

```python
# Parse standard files into nested dictionaries
qrels_dict = pytrec_eval.parse_qrel("qrels.txt")
run_dict = pytrec_eval.parse_run("run.txt")
```

---

## 2. `ir_measures` Provider

`trec_eval.compat.ir_measures` provides a provider adapter for the `ir_measures` library:

```python
import ir_measures
from ir_measures import AP, RR, nDCG, P
from trec_eval.compat.ir_measures import TeRustProvider

# Register te-rust as an evaluation provider
ir_measures.register_provider(TeRustProvider())

# Evaluate using standard ir_measures workflows
results = ir_measures.calc_aggregate(
    [AP, nDCG@10, RR, P@5],
    qrels,
    run,
    provider="te_rust",
)
```
