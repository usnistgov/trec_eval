import pytest
from trec_eval import Evaluator


@pytest.mark.parametrize("measure", ["set_F.nan", "rbp.p=inf", "Rprec_mult.nan,0.5", "11pt_avg.1e400"])
def test_non_finite_parameter_is_refused(measure):
    with pytest.raises(ValueError, match="not a finite number"):
        Evaluator({"1": {"a": 1, "b": 0}}, measures=[measure])
