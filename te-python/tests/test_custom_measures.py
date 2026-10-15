import pytest
import trec_eval
from trec_eval import Evaluator, Measure, QueryState, register_measure


@pytest.fixture
def sample_data():
    qrels = {
        "q1": {"d1": 0, "d2": 1, "d3": 0},
        "q2": {"d1": 1, "d2": 1, "d3": 0},
    }
    run = {
        "q1": {"d1": 1.0, "d2": 2.0, "d3": 0.5},
        "q2": {"d1": 0.5, "d2": 1.5, "d3": 2.0},
    }
    return qrels, run


class ReciprocalRankAtK(Measure):
    """Custom class-based reciprocal rank at cutoff k."""

    def __init__(self, k: int = 10):
        super().__init__(name=f"rr@{k}")
        self.k = k

    def calc_query(self, query: QueryState) -> float:
        for rank, rel in enumerate(query.relevances[: self.k], start=1):
            if rel >= query.relevance_level:
                return 1.0 / rank
        return 0.0


@register_measure(name="p_at_2")
def precision_at_two(query: QueryState) -> float:
    """Custom decorator-based precision at rank 2."""
    rel_count = sum(1 for r in query.relevances[:2] if r >= query.relevance_level)
    return rel_count / 2.0


class BuggyMeasure(Measure):
    """Custom measure designed to trigger an exception for sandboxing test."""

    def __init__(self):
        super().__init__(name="buggy_metric")

    def calc_query(self, query: QueryState) -> float:
        if query.qid == "q2":
            raise ZeroDivisionError("Simulated calculation bug on query q2")
        return 1.0


def test_class_based_custom_measure(sample_data):
    qrels, run = sample_data
    rr5 = ReciprocalRankAtK(k=5)

    evaluator = Evaluator(qrels, measures=[rr5])
    res = evaluator.evaluate(run)

    assert len(res) == 2
    assert "rr@5" in res["q1"]
    # In q1: rankings are d2 (rel 1, rank 1), d1 (rel 0, rank 2), d3 (rel 0, rank 3) -> RR = 1.0
    assert res["q1"]["rr@5"] == 1.0
    # In q2: rankings are d3 (rel 0, rank 1), d2 (rel 1, rank 2), d1 (rel 1, rank 3) -> RR = 0.5
    assert res["q2"]["rr@5"] == 0.5
    assert res.aggregate()["rr@5"] == pytest.approx(0.75)


def test_decorator_based_custom_measure(sample_data):
    qrels, run = sample_data
    evaluator = Evaluator(qrels, measures=[precision_at_two])
    res = evaluator.evaluate(run)

    assert "p_at_2" in res.aggregate()
    # q1: top 2 are d2 (rel 1) and d1 (rel 0) -> 1/2 = 0.5
    assert res["q1"]["p_at_2"] == 0.5
    # q2: top 2 are d3 (rel 0) and d2 (rel 1) -> 1/2 = 0.5
    assert res["q2"]["p_at_2"] == 0.5


def test_mixed_builtin_and_custom_measures(sample_data):
    qrels, run = sample_data
    evaluator = Evaluator(
        qrels,
        measures=["map", "ndcg@10", ReciprocalRankAtK(k=5), precision_at_two],
    )
    res = evaluator.evaluate(run)
    agg = res.aggregate()

    assert "map" in agg
    assert "ndcg_cut_10" in agg
    assert "rr@5" in agg
    assert "p_at_2" in agg


def test_custom_measure_exception_sandboxing(sample_data):
    qrels, run = sample_data
    evaluator = Evaluator(qrels, measures=["map", BuggyMeasure()])

    with pytest.raises(ValueError) as exc_info:
        evaluator.evaluate(run)

    err_msg = str(exc_info.value)
    assert "EvaluationError" in err_msg
    assert "buggy_metric" in err_msg
    assert "q2" in err_msg
    assert "ZeroDivisionError" in err_msg


def test_custom_measure_significance_testing(sample_data):
    qrels, run1 = sample_data
    run2 = {
        "q1": {"d1": 2.0, "d2": 1.0, "d3": 0.5},
        "q2": {"d1": 0.5, "d2": 1.0, "d3": 2.0},
    }

    evaluator = Evaluator(qrels, measures=[ReciprocalRankAtK(k=5)])
    res1 = evaluator.evaluate(run1)
    res2 = evaluator.evaluate(run2)

    comp = res1.compare(res2, measure="rr@5", test="paired_t")
    assert comp.measure == "rr@5"
    assert comp.n == 2
    assert isinstance(comp.statistic, float)
    assert isinstance(comp.pvalue, float)
