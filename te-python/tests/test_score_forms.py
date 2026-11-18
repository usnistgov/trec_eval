import math

import pytest
from trec_eval import Evaluator

QRELS = {"t": {"a": 1, "b": 2, "c": 3}}


def test_run_file_scores_read_as_strtod_reads_them(tmp_path):
    run = tmp_path / "scores.run"
    run.write_text("t Q0 a 1 5 r\nt Q0 b 2 inf r\nt Q0 c 3 1.5x r\n")
    with pytest.warns(UserWarning) as caught:
        result = Evaluator(QRELS, measures=["relstring"]).evaluate(str(run))
    messages = [str(w.message) for w in caught]
    assert any("score '1.5x' partially read as '1.5'" in m for m in messages), messages
    # b (inf) ranks first, then a (5) and c (1.5).
    assert result.per_query()["t"]["relstring"] == "'213'"


def test_a_run_file_score_out_of_range_warns(tmp_path):
    run = tmp_path / "range.run"
    run.write_text("t Q0 a 1 5 r\nt Q0 b 2 1e400 r\nt Q0 c 3 1e-400 r\n")
    with pytest.warns(UserWarning) as caught:
        result = Evaluator(QRELS, measures=["relstring"]).evaluate(str(run))
    messages = "\n".join(str(w.message) for w in caught)
    assert "score '1e400' out of range, read as inf" in messages
    assert "score '1e-400' out of range, read as 0" in messages
    assert result.per_query()["t"]["relstring"] == "'213'"


@pytest.mark.parametrize("score", [math.inf, -math.inf])
def test_infinite_scores_are_accepted_from_a_dict(score):
    run = {"t": {"a": 5.0, "b": score, "c": 3.0}}
    result = Evaluator(QRELS, measures=["relstring"]).evaluate(run)
    assert result.per_query()["t"]["relstring"] == ("'213'" if score > 0 else "'132'")


def test_nan_scores_are_refused_from_a_file_and_a_dict(tmp_path):
    run = tmp_path / "nan.run"
    run.write_text("t Q0 a 1 5 r\nt Q0 b 2 nan r\n")
    evaluator = Evaluator(QRELS, measures=["map"])
    with pytest.raises(ValueError, match="invalid float for field 'sim': 'nan'"):
        evaluator.evaluate(str(run))
    with pytest.raises(ValueError, match="Score of document 'b' for query 't' is NaN"):
        evaluator.evaluate({"t": {"a": 5.0, "b": math.nan}})
