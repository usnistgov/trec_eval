"""
Drop-in compatibility layer for `pytrec_eval`.

Existing scripts using `pytrec_eval` can switch to `te-rust` simply by:
```python
import trec_eval.compat.pytrec_eval as pytrec_eval
```
"""

import collections
import io
from typing import Any, Dict, Iterable, Set, Union
from trec_eval._trec_eval import Evaluator, __version__


# Known measures supported by pytrec_eval
supported_measures: Set[str] = {
    "official", "all_trec", "set",
    "runid", "num_ret", "num_rel", "num_rel_ret", "num_nonrel_judged_ret",
    "map", "map_cut", "gm_map", "Rprec", "Rprec_mult", "recip_rank",
    "P", "relative_P", "bpref", "gm_bpref", "binG", "G",
    "ndcg", "ndcg_cut", "ndcg_rel", "ndcg_p", "Rndcg",
    "recall", "success", "11pt_avg", "iprec_at_recall", "utility", "relstring",
    "set_relative_P", "set_map", "set_F", "infap", "unj", "rbp", "rbp_resid", "yaap",
}


def parse_run(f_or_path: Union[str, io.IOBase, Any]) -> Dict[str, Dict[str, float]]:
    """Parse a TREC run file or file-like stream into a nested dictionary."""
    run: Dict[str, Dict[str, float]] = collections.defaultdict(dict)

    if isinstance(f_or_path, str) and "\n" not in f_or_path and len(f_or_path) < 1024:
        with open(f_or_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 6 and not parts[0].startswith("#"):
                    qid, docno, score = parts[0], parts[2], float(parts[4])
                    run[qid][docno] = score
        return dict(run)

    # File-like or text content
    lines = f_or_path.splitlines() if isinstance(f_or_path, str) else f_or_path
    for line in lines:
        line_str = line.decode("utf-8") if isinstance(line, bytes) else str(line)
        parts = line_str.strip().split()
        if len(parts) >= 6 and not parts[0].startswith("#"):
            qid, docno, score = parts[0], parts[2], float(parts[4])
            run[qid][docno] = score

    return dict(run)


def parse_qrel(f_or_path: Union[str, io.IOBase, Any]) -> Dict[str, Dict[str, int]]:
    """Parse a TREC qrels file or file-like stream into a nested dictionary."""
    qrel: Dict[str, Dict[str, int]] = collections.defaultdict(dict)

    if isinstance(f_or_path, str) and "\n" not in f_or_path and len(f_or_path) < 1024:
        with open(f_or_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 4 and not parts[0].startswith("#"):
                    qid, docno, rel = parts[0], parts[2], int(parts[3])
                    qrel[qid][docno] = rel
        return dict(qrel)

    lines = f_or_path.splitlines() if isinstance(f_or_path, str) else f_or_path
    for line in lines:
        line_str = line.decode("utf-8") if isinstance(line, bytes) else str(line)
        parts = line_str.strip().split()
        if len(parts) >= 4 and not parts[0].startswith("#"):
            qid, docno, rel = parts[0], parts[2], int(parts[3])
            qrel[qid][docno] = rel

    return dict(qrel)


class RelevanceEvaluator:
    """Drop-in replacement for pytrec_eval.RelevanceEvaluator."""

    def __init__(
        self,
        qrel: Any,
        measures: Union[Set[str], Iterable[str], str],
        relevance_level: int = 1,
    ):
        self._evaluator = Evaluator(
            qrels=qrel,
            measures=measures,
            relevance_level=relevance_level,
        )
        self.measures = set(measures) if not isinstance(measures, str) else {measures}

    def evaluate(self, run: Any) -> Dict[str, Dict[str, float]]:
        """Evaluate run and return nested dict matching pytrec_eval output format."""
        eval_result = self._evaluator.evaluate(run)
        return eval_result.to_dict()


__all__ = [
    "RelevanceEvaluator",
    "parse_run",
    "parse_qrel",
    "supported_measures",
    "__version__",
]
