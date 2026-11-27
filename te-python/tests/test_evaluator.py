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


def test_parameter_validation_and_sorting(sample_qrels_dict, sample_run_dict):
    # Unsorted cutoffs match ascending cutoffs
    asc = evaluate(sample_qrels_dict, sample_run_dict, measures=["ndcg_cut.1,3", "map_cut.1,3"])
    desc = evaluate(sample_qrels_dict, sample_run_dict, measures=["ndcg_cut.3,1", "map_cut.3,1"])
    assert desc.aggregate() == pytest.approx(asc.aggregate())

    # Invalid measure specifications / parameters raise ValueError
    for bad in ["P.", "P.0", "P.5,5", "ndcg.1=x", "rbp.p=1.0", "rbp_resid.1=2", "set_F.-0.5"]:
        with pytest.raises(ValueError):
            Evaluator(sample_qrels_dict, measures=[bad])

    # Invalid global_gains raises ValueError
    with pytest.raises(ValueError):
        Evaluator(sample_qrels_dict, measures=["ndcg"], global_gains="1=x")


def test_run_score_validation_issue_107(tmp_path):
    qrels_path = tmp_path / "abc.qrels"
    qrels_path.write_text("1 0 a 1\n1 0 b 2\n1 0 c 3\n")
    ok_run = tmp_path / "ok.run"
    ok_run.write_text("1 Q0 a 1 5 r\n1 Q0 b 2 4 r\n1 Q0 c 3 3 r\n")

    ev = Evaluator(str(qrels_path), measures=["relstring"])
    res = ev.evaluate(str(ok_run))
    assert res["1"]["relstring"] == "'123'"

    # Run files with non-numeric, trailing-garbage, NaN, or Inf scores raise ValueError
    for bad in ["nan", "abc", "1.5x", "inf", "-inf", "1e400"]:
        bad_run = tmp_path / f"{bad}.run"
        bad_run.write_text(f"1 Q0 a 1 5 r\n1 Q0 b 2 {bad} r\n1 Q0 c 3 3 r\n")
        with pytest.raises(ValueError):
            ev.evaluate(str(bad_run))

    # In-memory dict and iterable inputs reject NaN and Inf with ValueError
    for bad_float in [float("nan"), float("inf"), float("-inf")]:
        with pytest.raises(ValueError):
            ev.evaluate({"1": {"a": 5.0, "b": bad_float, "c": 3.0}})
        with pytest.raises(ValueError):
            ev.evaluate([("1", "a", 5.0), ("1", "b", bad_float), ("1", "c", 3.0)])
        with pytest.raises(ValueError):
            ev.evaluate([
                ScoredDoc("1", "a", 5.0),
                ScoredDoc("1", "b", bad_float),
                ScoredDoc("1", "c", 3.0),
            ])

    # Non-numeric strings in dict/iterable raise TypeError
    for bad_str in ["abc", "1.5x"]:
        with pytest.raises(TypeError):
            ev.evaluate({"1": {"a": 5.0, "b": bad_str, "c": 3.0}})


def test_sparse_rel_levels_and_extreme_relevance_level_issue_102():
    qrels = {
        "1": {"a": 2, "b": 0, "c": 1, "d": 2},
        "2": {"a": 1, "b": 0},
    }
    run = {
        "1": {"a": 4.0, "c": 3.0, "b": 2.0, "d": 1.0},
        "2": {"a": 2.0, "b": 1.0},
    }

    # Issue #102: huge relevance_level must return 0.0 immediately and leave rbp unchanged
    ev_high = Evaluator(qrels, measures=["bpref", "gm_bpref", "rbp"], relevance_level=10**12)
    res_high = ev_high.evaluate(run)
    assert res_high["1"]["bpref"] == 0.0
    assert res_high["2"]["bpref"] == 0.0
    assert res_high.aggregate()["bpref"] == 0.0
    assert res_high.aggregate()["gm_bpref"] == pytest.approx(0.00001)
    assert res_high["2"]["rbp"] == pytest.approx(0.1)

    # Negative relevance_level treats all judged (>= 0) docs as relevant
    ev_neg = Evaluator(qrels, measures=["bpref", "gm_bpref"], relevance_level=-1)
    res_neg = ev_neg.evaluate(run)
    assert res_neg.aggregate()["bpref"] == pytest.approx(1.0)
    assert res_neg.aggregate()["gm_bpref"] == pytest.approx(1.0)

    # Sparse huge qrels grades (including i64::MAX) work without excessive allocation
    huge_qrels = {"1": {"d1": 0, "d2": 10**9, "d3": (1 << 63) - 1}}
    huge_run = {"1": {"d3": 3.0, "d2": 2.0, "d1": 1.0}}
    res_huge = evaluate(huge_qrels, huge_run, measures=["map", "ndcg", "bpref"])
    assert res_huge["1"]["map"] == pytest.approx(1.0)
    assert res_huge["1"]["ndcg"] == pytest.approx(1.0)
    assert res_huge["1"]["bpref"] == pytest.approx(1.0)

    # Issue #94: rbp rescales gains from 0.0
    res_rbp = evaluate({"1": {"a": 1, "b": 0}}, {"1": {"a": 2.0, "b": 1.0}}, measures=["rbp.0=1,1=3"])
    assert res_rbp.aggregate()["rbp_0=1,1=3"] == pytest.approx(0.13)

