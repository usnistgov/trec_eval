import os
from pathlib import Path
import pytest
import trec_eval.compat.pytrec_eval as pytrec_eval

TEST_DATA_DIR = Path(__file__).resolve().parent.parent.parent / "te-rust" / "tests" / "test-data"
QRELS_FILE = str(TEST_DATA_DIR / "qrels.test")
RESULTS_FILE = str(TEST_DATA_DIR / "results.test")


def test_pytrec_eval_drop_in_relevance_evaluator():
    qrels = {
        "q1": {"d1": 0, "d2": 1, "d3": 0},
        "q2": {"d2": 1, "d3": 1},
    }
    run = {
        "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
        "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
    }

    evaluator = pytrec_eval.RelevanceEvaluator(qrels, {"map", "ndcg"})
    res = evaluator.evaluate(run)

    # Returns exact nested dict
    assert isinstance(res, dict)
    assert isinstance(res["q1"], dict)
    assert "map" in res["q1"]
    assert "ndcg" in res["q1"]
    assert res["q1"]["map"] == pytest.approx(1.0 / 3.0)
    assert res["q1"]["ndcg"] == pytest.approx(0.5)


def test_pytrec_eval_parse_helpers():
    assert os.path.exists(QRELS_FILE)
    assert os.path.exists(RESULTS_FILE)

    qrels_dict = pytrec_eval.parse_qrel(QRELS_FILE)
    run_dict = pytrec_eval.parse_run(RESULTS_FILE)

    assert isinstance(qrels_dict, dict)
    assert isinstance(run_dict, dict)
    assert len(qrels_dict) > 0
    assert len(run_dict) > 0

    evaluator = pytrec_eval.RelevanceEvaluator(qrels_dict, {"map", "recip_rank"})
    res = evaluator.evaluate(run_dict)
    assert len(res) > 0


def test_pytrec_eval_supported_measures():
    assert "map" in pytrec_eval.supported_measures
    assert "ndcg" in pytrec_eval.supported_measures
    assert "recip_rank" in pytrec_eval.supported_measures
    assert "official" in pytrec_eval.supported_measures


def test_pytrec_eval_rejects_invalid_run_scores(tmp_path):
    qrels = {"1": {"a": 1, "b": 2, "c": 3}}
    evaluator = pytrec_eval.RelevanceEvaluator(qrels, {"map"})

    for bad in ["nan", "abc", "1.5x", "inf", "-inf"]:
        run_text = f"1 Q0 a 1 5 r\n1 Q0 b 2 {bad} r\n1 Q0 c 3 3 r\n"
        with pytest.raises(ValueError):
            pytrec_eval.parse_run(run_text)

        run_file = tmp_path / f"{bad}.run"
        run_file.write_text(run_text)
        with pytest.raises(ValueError):
            pytrec_eval.parse_run(str(run_file))

    for bad_float in [float("nan"), float("inf"), float("-inf")]:
        with pytest.raises(ValueError):
            evaluator.evaluate({"1": {"a": 5.0, "b": bad_float, "c": 3.0}})

