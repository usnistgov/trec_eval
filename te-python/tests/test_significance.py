import pytest
import trec_eval
from trec_eval import Evaluator, adjust_pvalues


@pytest.fixture
def multi_system_setup():
    qrels = {
        f"q{i}": {f"d{j}": (1 if (i + j) % 3 == 0 else 0) for j in range(10)}
        for i in range(20)
    }

    # Baseline system: mixed ranking based on even/odd document IDs
    baseline = {
        f"q{i}": {f"d{j}": (0.8 if j % 2 == 0 else 0.3) for j in range(10)}
        for i in range(20)
    }

    # Strong system: all relevant documents ranked top
    strong = {
        f"q{i}": {f"d{j}": (1.5 if (i + j) % 3 == 0 else 0.1) for j in range(10)}
        for i in range(20)
    }

    # Weak system: relevant documents ranked bottom
    weak = {
        f"q{i}": {f"d{j}": (0.1 if (i + j) % 3 == 0 else 1.5) for j in range(10)}
        for i in range(20)
    }

    return qrels, baseline, strong, weak



def test_single_pair_comparison_against_scipy(multi_system_setup):
    pytest.importorskip("scipy.stats")
    from scipy.stats import ttest_rel

    qrels, baseline, strong, _ = multi_system_setup
    evaluator = Evaluator(qrels, measures=["map"])

    res_base = evaluator.evaluate(baseline)
    res_strong = evaluator.evaluate(strong)

    comp = res_strong.compare(res_base, measure="map", test="paired_t")

    scores_strong = res_strong.to_numpy("map")
    scores_base = res_base.to_numpy("map")
    scipy_res = ttest_rel(scores_strong, scores_base)

    assert comp.statistic == pytest.approx(scipy_res.statistic, rel=1e-4)
    assert comp.pvalue == pytest.approx(scipy_res.pvalue, rel=1e-4)
    assert comp.significant(0.05)


def test_permutation_and_bootstrap_tests(multi_system_setup):
    qrels, baseline, strong, _ = multi_system_setup
    evaluator = Evaluator(qrels, measures=["map"])

    res_base = evaluator.evaluate(baseline)
    res_strong = evaluator.evaluate(strong)

    # Permutation test reproducibility with seed
    perm1 = res_strong.compare(res_base, measure="map", test="permutation", num_resamples=5000, seed=123)
    perm2 = res_strong.compare(res_base, measure="map", test="permutation", num_resamples=5000, seed=123)
    assert perm1.pvalue == pytest.approx(perm2.pvalue)
    assert perm1.pvalue < 0.05

    # Bootstrap test reproducibility with seed
    boot1 = res_strong.compare(res_base, measure="map", test="bootstrap", num_resamples=5000, seed=456)
    boot2 = res_strong.compare(res_base, measure="map", test="bootstrap", num_resamples=5000, seed=456)
    assert boot1.pvalue == pytest.approx(boot2.pvalue)
    assert boot1.pvalue < 0.05


def test_multiple_comparisons_adjustments():
    raw_p = [0.01, 0.04, 0.03, 0.20]

    # Bonferroni (m = 4): [0.04, 0.16, 0.12, 0.80]
    bonf = adjust_pvalues(raw_p, method="bonferroni")
    assert bonf == pytest.approx([0.04, 0.16, 0.12, 0.80])

    # Holm-Bonferroni (sorted: 0.01*4=0.04, 0.03*3=0.09, 0.04*2=0.09 (monotonic), 0.20*1=0.20)
    # in original order: index 0 (0.01) -> 0.04, index 1 (0.04) -> 0.09, index 2 (0.03) -> 0.09, index 3 (0.20) -> 0.20
    holm = adjust_pvalues(raw_p, method="holm")
    assert holm == pytest.approx([0.04, 0.09, 0.09, 0.20])

    # Benjamini-Hochberg FDR
    fdr = adjust_pvalues(raw_p, method="fdr_bh")
    assert all(0.0 <= p <= 1.0 for p in fdr)

    # None
    assert adjust_pvalues(raw_p, method="none") == raw_p


def test_compare_against_baseline(multi_system_setup):
    qrels, baseline, strong, weak = multi_system_setup
    evaluator = Evaluator(qrels, measures=["map", "ndcg@10"])

    comp_table = evaluator.compare_against_baseline(
        baseline=baseline,
        candidates={
            "strong_model": strong,
            "weak_model": weak,
        },
        measures=["map"],
        test="paired_t",
        correction="holm",
        alpha=0.05,
    )

    assert len(comp_table) == 2
    rows = comp_table.to_dict()
    assert len(rows) == 2
    assert rows[0]["candidate"] == "strong_model"
    assert rows[0]["significant"] is True

    # Representation and dataframe
    repr_str = repr(comp_table)
    assert "strong_model" in repr_str

    try:
        import pandas as pd
        df = comp_table.to_dataframe()
        assert len(df) == 2
        assert "p_adj" in df.columns
    except ImportError:
        with pytest.raises(ImportError):
            comp_table.to_dataframe()


def test_compare_all_pairs_matrix(multi_system_setup):
    qrels, baseline, strong, weak = multi_system_setup
    evaluator = Evaluator(qrels, measures=["map"])

    matrix = evaluator.compare_all(
        runs={
            "baseline": baseline,
            "strong": strong,
            "weak": weak,
        },
        measure="map",
        test="paired_t",
        correction="fdr_bh",
        alpha=0.05,
    )

    assert matrix.systems == ["baseline", "strong", "weak"]
    assert "baseline" in matrix.pvalues_adjusted
    assert matrix.pvalues_adjusted["baseline"]["baseline"] == 1.0
    assert matrix.pvalues_adjusted["strong"]["weak"] < 0.05

    summary = matrix.summary_table()
    assert "Pairwise Comparisons Matrix" in summary
    assert "strong" in summary

    try:
        import pandas as pd
        df = matrix.to_dataframe()
        assert df.shape == (3, 3)
    except ImportError:
        with pytest.raises(ImportError):
            matrix.to_dataframe()

