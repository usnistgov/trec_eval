"""
Custom evaluation measures framework for trec_eval.

Allows defining user-defined IR evaluation measures in Python via subclassing
or functional decorators, participating seamlessly alongside built-in measures.
"""

from typing import Callable, List, Optional, Union
from trec_eval._trec_eval import QueryState


class Measure:
    """Base class for user-defined Python evaluation measures.

    Parameters
    ----------
    name : str
        Unique name for the measure (e.g. 'rr@5', 'custom_ndcg').
    """

    def __init__(self, name: str):
        self.name = name

    def calc_query(self, query: QueryState) -> Union[float, int, str]:
        """Calculate the score for a single query topic.

        Parameters
        ----------
        query : QueryState
            Aligned query state containing .relevances, .num_rel, .num_ret, etc.

        Returns
        -------
        float or int
            Score for this query.
        """
        raise NotImplementedError("Custom measures must implement calc_query(query: QueryState)")

    def aggregate(self, scores: List[float]) -> float:
        """Calculate the final summary score from the per-query scores.

        Parameters
        ----------
        scores : List[float]
            List of evaluated per-query scores across all topics included in
            the evaluation (including empty-ranking scores for missing qrels
            topics when ``complete_set_average=True``).

        Returns
        -------
        float
            Summary aggregated score across all queries.
        """
        if not scores:
            return 0.0
        return sum(scores) / float(len(scores))

    def __call__(self, query: QueryState) -> Union[float, int, str]:
        return self.calc_query(query)

    def __repr__(self) -> str:
        return f"<Measure: '{self.name}'>"


def register_measure(name: Optional[str] = None) -> Callable:
    """Decorator to register a custom evaluation function as a Measure.

    Parameters
    ----------
    name : str, optional
        Custom name for the measure. If omitted, uses the function's __name__.

    Examples
    --------
    >>> @register_measure(name="p_at_3")
    ... def precision_at_three(query: QueryState) -> float:
    ...     rel_count = sum(1 for r in query.relevances[:3] if r >= query.relevance_level)
    ...     return rel_count / 3.0
    """
    def decorator(fn: Callable[[QueryState], Union[float, int, str]]) -> Measure:
        measure_name = name or fn.__name__

        class FunctionMeasure(Measure):
            def __init__(self):
                super().__init__(name=measure_name)

            def calc_query(self, query: QueryState) -> Union[float, int, str]:
                return fn(query)

        return FunctionMeasure()

    return decorator
