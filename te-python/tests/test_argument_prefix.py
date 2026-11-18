import pytest
from trec_eval import Evaluator


def test_cutoff_reads_its_leading_integer_with_a_warning():
    qrels = {"1": {"a": 1, "b": 0}}
    with pytest.warns(UserWarning, match="value '5x' partially read as '5'"):
        evaluator = Evaluator(qrels, measures=["P.5x"])
    # One relevant document in the top 5.
    assert evaluator.evaluate({"1": {"a": 2.0, "b": 1.0}}).aggregate()["P_5"] == 0.2


def test_cutoff_without_a_leading_integer_is_refused():
    with pytest.raises(ValueError):
        Evaluator({"1": {"a": 1}}, measures=["P.x"])
