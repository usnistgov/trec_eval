import pytest
from trec_eval import Evaluator

QRELS = "1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n2 0 d1 0\n2 0 d2 1\n"


def test_a_repeated_judgment_is_refused_when_its_topic_is_scored(tmp_path):
    qrels = tmp_path / "repeat.qrels"
    qrels.write_text(QRELS + "3 0 x 1\n3 0 x 0\n")
    # Reading the file no longer fails: topic 3 has a repeat, but only -c would score it.
    evaluator = Evaluator(str(qrels), measures=["map"])
    run = {"1": {"d1": 3.0, "d2": 2.0, "d3": 1.0}, "2": {"d2": 2.0, "d1": 1.0}}
    assert evaluator.evaluate(run).aggregate()["map"] == pytest.approx(0.9167, abs=1e-4)
    complete = Evaluator(str(qrels), measures=["map"], complete_set_average=True)
    with pytest.raises(ValueError, match="qrels: duplicate document 'x' for query '3'"):
        complete.evaluate(run)


def test_a_repeated_run_document_is_refused_within_max_docs(tmp_path):
    qrels = tmp_path / "two.qrels"
    qrels.write_text(QRELS)
    query_ids = ["1", "1", "1", "2", "2"]
    doc_ids = ["d1", "d2", "d2", "d2", "d1"]
    scores = [3.0, 2.0, 1.0, 2.0, 1.0]
    with pytest.raises(ValueError, match="run: duplicate document 'd2' for query '1'"):
        Evaluator(str(qrels), measures=["map"]).evaluate_arrays(
            query_ids, doc_ids, scores
        )
    # With max_docs_per_topic=2 the second d2 is cut, as trec_eval -M 2 does: map 0.7500.
    cut = Evaluator(str(qrels), measures=["map"], max_docs_per_topic=2)
    result = cut.evaluate_arrays(query_ids, doc_ids, scores)
    assert result.aggregate()["map"] == pytest.approx(0.75)
    # num_q reads no ranking, so trec_eval does not check the repeat.
    counted = Evaluator(str(qrels), measures=["num_q"])
    assert counted.evaluate_arrays(query_ids, doc_ids, scores).aggregate()["num_q"] == 2
