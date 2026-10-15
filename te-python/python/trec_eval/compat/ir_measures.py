"""
`ir_measures` provider adapter for `te-rust`.

Allows registering `te-rust` as an acceleration provider in the `ir_measures` ecosystem:
```python
import ir_measures
from trec_eval.compat.ir_measures import TeRustProvider

ir_measures.register_provider(TeRustProvider())
```
"""

from typing import Any, Dict, Iterator, List, NamedTuple, Optional
from trec_eval._trec_eval import Evaluator


try:
    import ir_measures
    from ir_measures import Metric, measures, providers
    from ir_measures.providers.base import Any as AnyChoice, Choices, NOT_PROVIDED
    _HAS_IR_MEASURES = True
except ImportError:
    _HAS_IR_MEASURES = False
    providers = object
    Metric = object


def _build_measure_str(measure: Any) -> Optional[str]:
    """Convert an ir_measures Metric specification into a trec_eval measure string."""
    name = getattr(measure, "NAME", str(measure))
    if name == "P":
        cutoff = getattr(measure, "cutoff", None)
        return f"P_{cutoff}" if cutoff is not None else "P"
    elif name == "RR":
        return "recip_rank"
    elif name == "Rprec":
        return "Rprec"
    elif name == "AP":
        cutoff = getattr(measure, "cutoff", None)
        return f"map_cut_{cutoff}" if cutoff is not None else "map"
    elif name == "nDCG":
        cutoff = getattr(measure, "cutoff", None)
        return f"ndcg_cut_{cutoff}" if cutoff is not None else "ndcg"
    elif name == "R":
        cutoff = getattr(measure, "cutoff", None)
        return f"recall_{cutoff}" if cutoff is not None else "recall"
    elif name == "Bpref":
        return "bpref"
    elif name == "NumRet":
        return "num_ret"
    elif name == "NumRel":
        return "num_rel"
    elif name == "Success":
        cutoff = getattr(measure, "cutoff", None)
        return f"success_{cutoff}" if cutoff is not None else "success"
    return name.lower()


class TeRustEvaluator:
    """Evaluator adapter satisfying ir_measures evaluator protocol."""

    def __init__(self, measures_list: List[Any], qrels: Any):
        self.measures = measures_list
        # Translate ir_measures objects to trec_eval measure names
        self.measure_map: Dict[str, Any] = {}
        trec_eval_names = []
        for m in measures_list:
            meas_str = _build_measure_str(m)
            if meas_str:
                trec_eval_names.append(meas_str)
                self.measure_map[meas_str] = m

        self.evaluator = Evaluator(qrels, measures=trec_eval_names)

    def iter_calc(self, run: Any) -> Iterator[Any]:
        """Yield ir_measures Metric namedtuples for each query score."""
        eval_result = self.evaluator.evaluate(run)
        for qid, query_scores in eval_result.per_query().items():
            for meas_str, val in query_scores.items():
                if meas_str in self.measure_map:
                    meas_obj = self.measure_map[meas_str]
                    if _HAS_IR_MEASURES:
                        yield Metric(query_id=qid, measure=meas_obj, value=val)
                    else:
                        yield (qid, meas_obj, val)

    def calc_aggregate(self, run: Any) -> Dict[Any, float]:
        """Return aggregate summary metrics matching ir_measures."""
        eval_result = self.evaluator.evaluate(run)
        agg_scores = eval_result.aggregate()
        results = {}
        for meas_str, val in agg_scores.items():
            if meas_str in self.measure_map:
                meas_obj = self.measure_map[meas_str]
                results[meas_obj] = val
        return results


class TeRustProvider(providers.Provider if _HAS_IR_MEASURES else object):
    """ir_measures Provider using high-speed te-rust backend."""

    NAME = "te_rust"

    def __init__(self):
        if _HAS_IR_MEASURES:
            super().__init__()

    def _evaluator(self, measures_list: List[Any], qrels: Any) -> TeRustEvaluator:
        return TeRustEvaluator(measures_list, qrels)


def register():
    """Register TeRustProvider with ir_measures if installed."""
    if _HAS_IR_MEASURES:
        ir_measures.register_provider(TeRustProvider())


__all__ = [
    "TeRustProvider",
    "TeRustEvaluator",
    "register",
]
