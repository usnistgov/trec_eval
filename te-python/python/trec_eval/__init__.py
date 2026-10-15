"""
`trec_eval` Python interface powered by `te-rust`.
"""

from typing import Any, Dict, Iterable, List, NamedTuple, Optional, Set, Union

from trec_eval._trec_eval import ComparisonResult, EvalResult, Evaluator as _RustEvaluator, QueryState, __version__
from trec_eval.measures import Measure, register_measure
from trec_eval.stats import AllPairsMatrix, ComparisonRow, MultiComparisonTable, adjust_pvalues

ScoredDoc = NamedTuple('ScoredDoc', [('query_id', str), ('doc_id', str), ('score', float)])

Qrel = NamedTuple('Qrel', [('query_id', str), ('doc_id', str), ('relevance', int)])


class Evaluator(_RustEvaluator):
    """Core evaluation engine holding pre-indexed relevance judgments and evaluation configuration."""

    def compare_against_baseline(
        self,
        baseline: Any,
        candidates: Dict[str, Any],
        measures: Optional[Union[str, List[str]]] = None,
        test: str = "paired_t",
        correction: str = "holm",
        alpha: float = 0.05,
        num_resamples: int = 10000,
        seed: Optional[int] = None,
    ) -> MultiComparisonTable:
        """Compare multiple candidate runs against a baseline run with multiple testing correction.

        Parameters
        ----------
        baseline : run input
            The baseline system run.
        candidates : Dict[str, run input]
            Mapping of candidate name to candidate run.
        measures : str or list of str, optional
            Evaluation measure(s) to test. Defaults to all measures configured on this Evaluator.
        test : str, default 'paired_t'
            'paired_t', 'permutation', or 'bootstrap'.
        correction : str, default 'holm'
            'holm', 'fdr_bh', 'bonferroni', or 'none'.
        alpha : float, default 0.05
            Significance threshold.
        num_resamples : int, default 10000
            Number of resamples for permutation / bootstrap tests.
        seed : int, optional
            RNG seed for reproducibility.

        Returns
        -------
        MultiComparisonTable
        """
        baseline_res = self.evaluate(baseline)
        target_measures = [measures] if isinstance(measures, str) else (measures or self.measure_names)

        raw_rows = []
        for m in target_measures:
            base_mean = baseline_res.aggregate().get(m, 0.0)
            cand_results = []
            for name, cand_run in candidates.items():
                cand_res = self.evaluate(cand_run)
                cand_mean = cand_res.aggregate().get(m, 0.0)
                comp = cand_res.compare(baseline_res, measure=m, test=test, num_resamples=num_resamples, seed=seed)
                cand_results.append((name, m, base_mean, cand_mean, comp.mean_diff, comp.statistic, comp.pvalue))

            raw_pvals = [x[6] for x in cand_results]
            adj_pvals = adjust_pvalues(raw_pvals, method=correction)

            for item, adj_p in zip(cand_results, adj_pvals):
                name, m_name, b_score, c_score, diff, stat, p_raw = item
                raw_rows.append(ComparisonRow(
                    candidate=name,
                    measure=m_name,
                    baseline_score=b_score,
                    candidate_score=c_score,
                    diff=diff,
                    statistic=stat,
                    p_raw=p_raw,
                    p_adj=adj_p,
                    significant=(adj_p < alpha),
                ))

        return MultiComparisonTable(raw_rows, alpha=alpha, test=test, correction=correction)

    def compare_all(
        self,
        runs: Dict[str, Any],
        measure: str = "map",
        test: str = "paired_t",
        correction: str = "fdr_bh",
        alpha: float = 0.05,
        num_resamples: int = 10000,
        seed: Optional[int] = None,
    ) -> AllPairsMatrix:
        """Compute full pairwise comparisons matrix across multiple systems."""
        system_names = list(runs.keys())
        eval_results = {name: self.evaluate(run) for name, run in runs.items()}

        pvalues_raw: Dict[str, Dict[str, float]] = {s1: {} for s1 in system_names}
        mean_diffs: Dict[str, Dict[str, float]] = {s1: {} for s1 in system_names}

        pairs = []
        raw_pvals = []
        for i, s1 in enumerate(system_names):
            pvalues_raw[s1][s1] = 1.0
            mean_diffs[s1][s1] = 0.0
            for j in range(i + 1, len(system_names)):
                s2 = system_names[j]
                comp = eval_results[s1].compare(eval_results[s2], measure=measure, test=test, num_resamples=num_resamples, seed=seed)
                pvalues_raw[s1][s2] = comp.pvalue
                pvalues_raw[s2][s1] = comp.pvalue
                mean_diffs[s1][s2] = comp.mean_diff
                mean_diffs[s2][s1] = -comp.mean_diff
                pairs.append((s1, s2))
                raw_pvals.append(comp.pvalue)

        adj_pvals = adjust_pvalues(raw_pvals, method=correction) if raw_pvals else []
        pvalues_adj: Dict[str, Dict[str, float]] = {s1: {s2: 1.0 for s2 in system_names} for s1 in system_names}

        for (s1, s2), adj_p in zip(pairs, adj_pvals):
            pvalues_adj[s1][s2] = adj_p
            pvalues_adj[s2][s1] = adj_p

        return AllPairsMatrix(
            systems=system_names,
            measure=measure,
            test=test,
            correction=correction,
            alpha=alpha,
            pvalues_raw=pvalues_raw,
            pvalues_adj=pvalues_adj,
            mean_diffs=mean_diffs,
        )


def evaluate(

    qrels: Any,
    run: Any,
    measures: Optional[Union[str, Iterable[str], Set[str], List[str]]] = None,
    relevance_level: int = 1,
    complete_set_average: bool = False,
    judged_docs_only: bool = False,
    max_docs_per_topic: Optional[int] = None,
    num_docs_in_coll: int = 0,
    global_gains: Optional[str] = None,
    qid: Optional[str] = None,
    docno: Optional[str] = None,
    rel: Optional[str] = None,
    score: Optional[str] = None,
) -> EvalResult:
    """Evaluate a run against relevance judgments using trec_eval.

    Parameters
    ----------
    qrels : str, os.PathLike, Dict[str, Dict[str, int]], DataFrame, or Iterable
        Relevance judgments (file path, nested dict, DataFrame, or row iterable).
    run : str, os.PathLike, Dict[str, Dict[str, float]], DataFrame, or Iterable
        System run results (file path, nested dict, DataFrame, or row iterable).
    measures : str or list of str, optional
        Measure names or groups (e.g. ['map', 'ndcg@10', 'P.5,10']). Defaults to 'official'.
    relevance_level : int, default 1
        Minimum relevance level at which a document is considered relevant (-l).
    complete_set_average : bool, default False
        Average over all queries in qrels instead of only retrieved ones (-c).
    judged_docs_only : bool, default False
        Remove all unjudged documents before evaluating (-J).
    max_docs_per_topic : int, optional
        Maximum number of documents per topic to evaluate (-M).
    num_docs_in_coll : int, default 0
        Number of documents in the collection (-N).
    global_gains : str, optional
        Global relevance-to-gain mapping (e.g. '1=3.5,2=9.0').
    qid : str, optional
        Column name for query IDs when evaluating DataFrames.
    docno : str, optional
        Column name for document IDs when evaluating DataFrames.
    rel : str, optional
        Column name for relevance grades when qrels is a DataFrame.
    score : str, optional
        Column name for retrieval scores when run is a DataFrame.

    Returns
    -------
    EvalResult
        Rich evaluation result with mapping protocol and .aggregate() / .per_query() methods.
    """
    evaluator = Evaluator(
        qrels=qrels,
        measures=measures,
        relevance_level=relevance_level,
        complete_set_average=complete_set_average,
        judged_docs_only=judged_docs_only,
        max_docs_per_topic=max_docs_per_topic,
        num_docs_in_coll=num_docs_in_coll,
        global_gains=global_gains,
        qid=qid,
        docno=docno,
        rel=rel,
    )
    return evaluator.evaluate(run, qid=qid, docno=docno, score=score)


from trec_eval import compat

__all__ = [

    "__version__",
    "Evaluator",
    "EvalResult",
    "ComparisonResult",
    "ComparisonRow",
    "MultiComparisonTable",
    "AllPairsMatrix",
    "adjust_pvalues",
    "Measure",
    "QueryState",
    "register_measure",
    "ScoredDoc",
    "Qrel",
    "evaluate",
    "compat",
]




