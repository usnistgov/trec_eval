import trec_eval
from trec_eval import ScoredDoc, Qrel, _trec_eval


def test_import_and_version():
    assert _trec_eval.__version__ == "11.0.0.dev0"


def test_namedtuples():
    doc = ScoredDoc(query_id="q1", doc_id="d1", score=1.5)
    assert doc.query_id == "q1"
    assert doc.doc_id == "d1"
    assert doc.score == 1.5

    qrel = Qrel(query_id="q1", doc_id="d1", relevance=1)
    assert qrel.query_id == "q1"
    assert qrel.doc_id == "d1"
    assert qrel.relevance == 1
