import pytest
from trec_eval import Evaluator


def test_fractional_grade_reads_as_its_leading_integer_with_a_warning(tmp_path):
    qrels = tmp_path / "grades.qrels"
    qrels.write_text("t 0 a 1.5\nt 0 b 0.9\nt 0 c 0\n")
    with pytest.warns(UserWarning) as caught:
        evaluator = Evaluator(str(qrels), measures=["map"])
    messages = [str(w.message) for w in caught]
    assert any("value '1.5' partially read as '1'" in m for m in messages), messages
    assert any("value '0.9' partially read as '0'" in m for m in messages), messages
    # Only a (1.5 read as 1) is relevant, at rank 2.
    run = {"t": {"b": 3.0, "a": 2.0, "c": 1.0}}
    assert evaluator.evaluate(run).aggregate()["map"] == 0.5
