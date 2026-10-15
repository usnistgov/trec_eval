import pytest
import trec_eval
from trec_eval import Evaluator, evaluate


class MockDataFrame:
    """Mock DataFrame implementing column access and .columns attribute."""
    def __init__(self, data: dict):
        self.data = data
        self.columns = list(data.keys())

    def __getitem__(self, col):
        return self.data[col]


def test_evaluate_arrays():
    qrels = {
        "q1": {"d1": 0, "d2": 1},
        "q2": {"d2": 1, "d3": 1},
    }
    evaluator = Evaluator(qrels, measures=["map", "recip_rank"])

    query_ids = ["q1", "q1", "q2", "q2"]
    doc_ids = ["d1", "d2", "d2", "d3"]
    scores = [0.1, 0.9, 0.8, 0.2]

    res = evaluator.evaluate_arrays(query_ids, doc_ids, scores)
    assert len(res) == 2
    assert res["q1"]["recip_rank"] == 1.0
    assert res["q1"]["map"] == 1.0


def test_dataframe_ingestion_default_columns():
    qrels_df = MockDataFrame({
        "query_id": ["q1", "q1", "q2", "q2"],
        "doc_id": ["d1", "d2", "d2", "d3"],
        "relevance": [0, 1, 1, 1],
    })

    run_df = MockDataFrame({
        "query_id": ["q1", "q1", "q2", "q2"],
        "doc_id": ["d1", "d2", "d2", "d3"],
        "score": [0.2, 1.5, 0.9, 0.3],
    })

    res = evaluate(qrels_df, run_df, measures=["map", "recip_rank"])
    assert len(res) == 2
    assert res["q1"]["recip_rank"] == 1.0


def test_dataframe_ingestion_custom_columns():
    qrels_df = MockDataFrame({
        "topic": ["q1", "q1", "q2"],
        "docno": ["d1", "d2", "d2"],
        "rel_grade": [0, 1, 1],
    })

    run_df = MockDataFrame({
        "topic": ["q1", "q1", "q2"],
        "docno": ["d1", "d2", "d2"],
        "similarity": [0.1, 0.8, 0.5],
    })

    evaluator = Evaluator(qrels_df, measures=["map"], qid="topic", docno="docno", rel="rel_grade")
    res = evaluator.evaluate(run_df, qid="topic", docno="docno", score="similarity")

    assert len(res) == 2
    assert res["q1"]["map"] == 1.0


def test_to_numpy_and_fallback():
    qrels = {"q1": {"d1": 1}, "q2": {"d2": 1}}
    run = {"q1": {"d1": 1.0}, "q2": {"d2": 0.5}}

    res = evaluate(qrels, run, measures=["map"])
    arr = res.to_numpy("map")

    # If numpy is installed, returns ndarray; if not, returns list of floats
    if hasattr(arr, "shape"):
        assert arr.shape == (2,)
        assert list(arr) == pytest.approx([1.0, 1.0])
    else:
        assert isinstance(arr, list)
        assert arr == pytest.approx([1.0, 1.0])


def test_to_dataframe_wide_and_tidy():
    pytest.importorskip("pandas")
    import pandas as pd

    qrels = {"q1": {"d1": 1, "d2": 0}, "q2": {"d2": 1}}
    run = {"q1": {"d1": 1.0, "d2": 0.2}, "q2": {"d2": 0.5}}

    res = evaluate(qrels, run, measures=["map", "recip_rank"])

    # Wide format
    df_wide = res.to_dataframe(format="wide")
    assert isinstance(df_wide, pd.DataFrame)
    assert "query_id" in df_wide.columns
    assert "map" in df_wide.columns
    assert "recip_rank" in df_wide.columns
    assert len(df_wide) == 2

    # Tidy format
    df_tidy = res.to_dataframe(format="tidy")
    assert isinstance(df_tidy, pd.DataFrame)
    assert set(df_tidy.columns) == {"query_id", "measure", "value"}
    assert len(df_tidy) == 4  # 2 queries x 2 measures
