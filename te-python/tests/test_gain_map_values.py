import pytest
from trec_eval import Evaluator

QRELS = {"t": {"a": 2, "b": 0, "c": 1, "d": 3}}
RUN = {"t": {"d": 5.0, "x": 4.0, "a": 3.0, "b": 2.0, "c": 1.0}}


@pytest.mark.parametrize(
    "measure", ["ndcg.1=x", "ndcg.x=1", "ndcg.5,5", "ndcg.1=2,1=5", "rbp.p=0.5,q=1"]
)
def test_unusable_gain_map_is_refused(measure):
    with pytest.raises(ValueError):
        Evaluator(QRELS, measures=[measure])


@pytest.mark.parametrize("gains", ["1=x", "x=1", "1=2,1=5"])
def test_unusable_global_gains_are_refused(gains):
    with pytest.raises(ValueError, match="Invalid global_gains"):
        Evaluator(QRELS, measures=["ndcg"], global_gains=gains)


def test_legal_global_gains_are_applied():
    # As `te-rust --global-gains 1=3,2=9 -m ndcg` and trec_eval's `ndcg.1=3,2=9`: 0.6988.
    evaluator = Evaluator(QRELS, measures=["ndcg"], global_gains="1=3,2=9")
    assert evaluator.evaluate(RUN).aggregate()["ndcg"] == pytest.approx(0.6988, abs=5e-5)
