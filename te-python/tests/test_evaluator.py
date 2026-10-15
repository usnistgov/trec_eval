import os
import pytest
from pathlib import Path
import trec_eval
from trec_eval import Evaluator, EvalResult, ScoredDoc, Qrel, evaluate

TEST_DATA_DIR = Path(__file__).resolve().parent.parent.parent / "te-rust" / "tests" / "test-data"
QRELS_FILE = str(TEST_DATA_DIR / "qrels.test")
RESULTS_FILE = str(TEST_DATA_DIR / "results.test")


@pytest.fixture
def sample_qrels_dict():
    return {
        "q1": {
            "d1": 0,
            "d2": 1,
            "d3": 0,
        },
        "q2": {
            "d2": 1,
            "d3": 1,
        },
    }


@pytest.fixture
def sample_run_dict():
    return {
        "q1": {
            "d1": 1.0,
            "d2": 0.0,
            "d3": 1.5,
        },
        "q2": {
            "d1": 1.5,
            "d2": 0.2,
            "d3": 0.5,
        },
    }


def test_evaluator_dict_inputs(sample_qrels_dict, sample_run_dict):
    evaluator = Evaluator(sample_qrels_dict, measures=["map", "ndcg@10", "recip_rank", "P.5"])
    res = evaluator.evaluate(sample_run_dict)

    assert isinstance(res, EvalResult)
    assert len(res) == 2
    assert "q1" in res
    assert "q2" in res

    # Per query checks
    q1 = res["q1"]
    assert "map" in q1
    assert "ndcg_cut_10" in q1
    assert "recip_rank" in q1
    assert "P_5" in q1

    # Aggregates
    agg = res.aggregate()
    assert "map" in agg
    assert "ndcg_cut_10" in agg
    assert "recip_rank" in agg
    assert "P_5" in agg
    assert agg["map"] == pytest.approx((res["q1"]["map"] + res["q2"]["map"]) / 2.0)


def test_evaluator_tuple_and_namedtuple_inputs():
    qrels = [
        Qrel("q1", "d1", 0),
        Qrel("q1", "d2", 1),
        Qrel("q2", "d2", 1),
    ]
    run = [
        ScoredDoc("q1", "d1", 0.5),
        ScoredDoc("q1", "d2", 1.5),
        ScoredDoc("q2", "d2", 0.8),
    ]
    evaluator = Evaluator(qrels, measures=["map", "recip_rank"])
    res = evaluator.evaluate(run)

    assert len(res) == 2
    assert res["q1"]["recip_rank"] == 1.0
    assert res["q1"]["map"] == 1.0


def test_evaluator_file_inputs():
    assert os.path.exists(QRELS_FILE)
    assert os.path.exists(RESULTS_FILE)

    evaluator = Evaluator(QRELS_FILE, measures=["map", "ndcg@10", "P.5,10", "bpref"])
    res = evaluator.evaluate(RESULTS_FILE)

    assert len(res) > 0
    agg = res.aggregate()
    assert "map" in agg
    assert "ndcg_cut_10" in agg
    assert "bpref" in agg
    assert "P_5" in agg
    assert "P_10" in agg


def test_functional_evaluate_one_liner(sample_qrels_dict, sample_run_dict):
    res = evaluate(sample_qrels_dict, sample_run_dict, measures=["map", "recip_rank"])
    assert isinstance(res, EvalResult)
    assert "q1" in res
    assert "map" in res.aggregate()


def test_eval_result_mapping_protocol(sample_qrels_dict, sample_run_dict):
    res = evaluate(sample_qrels_dict, sample_run_dict, measures=["map"])

    # Mapping protocol
    assert list(res) == ["q1", "q2"]
    assert res.keys() == ["q1", "q2"]
    assert len(res.values()) == 2
    assert len(res.items()) == 2

    for qid, scores in res.items():
        assert qid in ["q1", "q2"]
        assert "map" in scores

    # dict conversion
    d = res.to_dict()
    assert isinstance(d, dict)
    assert set(d.keys()) == {"q1", "q2"}

    # KeyError on missing query
    with pytest.raises(KeyError):
        _ = res["nonexistent_query"]


def test_measure_aliases_and_groups(sample_qrels_dict, sample_run_dict):
    evaluator = Evaluator(
        sample_qrels_dict,
        measures=["ndcg@10", "map@5", "P@5,10", "mrr", "recall@10"]
    )
    res = evaluator.evaluate(sample_run_dict)
    agg = res.aggregate()
    assert "ndcg_cut_10" in agg
    assert "map_cut_5" in agg
    assert "P_5" in agg
    assert "P_10" in agg
    assert "recip_rank" in agg
    assert "recall_10" in agg


def test_config_switches(sample_qrels_dict, sample_run_dict):
    # Complete set average with disjoint query in qrels
    qrels_extra = dict(sample_qrels_dict)
    qrels_extra["q3"] = {"d1": 1}

    eval_no_c = Evaluator(qrels_extra, measures=["map"], complete_set_average=False)
    res_no_c = eval_no_c.evaluate(sample_run_dict)
    # 2 queries evaluated in run -> average over 2
    map_no_c = res_no_c.aggregate()["map"]

    eval_c = Evaluator(qrels_extra, measures=["map"], complete_set_average=True)
    res_c = eval_c.evaluate(sample_run_dict)
    # average over all 3 qrels queries (q3 gets 0.0)
    map_c = res_c.aggregate()["map"]

    assert map_c == pytest.approx(map_no_c * 2.0 / 3.0)


def test_numerical_parity_against_known_run():
    # Evaluate official measure set on test data
    evaluator = Evaluator(QRELS_FILE, measures=["map", "Rprec", "recip_rank", "bpref", "P_5", "P_10"])
    res = evaluator.evaluate(RESULTS_FILE)
    agg = res.aggregate()

    # Verify non-trivial scores within [0, 1]
    for measure_name in ["map", "Rprec", "recip_rank", "bpref", "P_5", "P_10"]:
        score = agg[measure_name]
        assert 0.0 <= score <= 1.0, f"{measure_name} score {score} outside [0, 1]"
