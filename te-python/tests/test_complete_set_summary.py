from trec_eval import Evaluator, Measure, QueryState


class One(Measure):
    def __init__(self):
        super().__init__(name="one")

    def calc_query(self, query: QueryState) -> float:
        return 1.0


def test_complete_set_averages_builtin_and_custom_measures_without_overlap():
    qrels = {"q1": {"d1": 1, "d2": 0}, "q2": {"d1": 0, "d2": 1}}
    run = {"q3": {"d1": 1.0}}
    evaluator = Evaluator(qrels, measures=["map", "num_q", One()], complete_set_average=True)
    summary = evaluator.evaluate(run).aggregate()
    assert summary["num_q"] == 2
    assert summary["map"] == 0.0
    # Both qrels topics score 1, and -c divides by the two qrels topics.
    assert summary["one"] == 1.0
