import os
from pathlib import Path
import pytest
import trec_eval.compat.pytrec_eval as pytrec_eval
from trec_eval.compat.ir_measures import TeRustEvaluator

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


class MockIRMeasure:
    def __init__(self, name, cutoff=None):
        self.NAME = name
        self.cutoff = cutoff

    def __repr__(self):
        return f"{self.NAME}@{self.cutoff}" if self.cutoff else self.NAME


def test_ir_measures_evaluator_adapter():
    qrels = {
        "q1": {"d1": 0, "d2": 1, "d3": 0},
        "q2": {"d2": 1, "d3": 1},
    }
    run = {
        "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
        "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
    }

    measures = [
        MockIRMeasure("AP"),
        MockIRMeasure("nDCG", cutoff=10),
        MockIRMeasure("RR"),
    ]

    evaluator = TeRustEvaluator(measures, qrels)

    # Test aggregate
    agg = evaluator.calc_aggregate(run)
    assert len(agg) == 3

    # Test iter_calc
    items = list(evaluator.iter_calc(run))
    assert len(items) == 6  # 2 queries x 3 measures
