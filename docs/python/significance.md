# Statistical Significance Testing & Multiple Testing Corrections

In Information Retrieval experimentation, comparing system effectiveness across topic queries requires statistical hypothesis testing. Evaluating multiple candidates or measures also requires adjusting p-values to control family-wise error rates or false discovery rates.

---

## 1. Single Pair Comparison (`res_a.compare`)

Given two evaluation results evaluated against the same judgments, `.compare()` performs a paired hypothesis test on aligned topic score vectors:

```python
import trec_eval

evaluator = trec_eval.Evaluator("qrels.txt", measures=["map", "ndcg@10"])

res_baseline = evaluator.evaluate("runs/bm25.txt")
res_splade = evaluator.evaluate("runs/splade.txt")

# Paired Student's t-test on MAP
comp = res_splade.compare(res_baseline, measure="map", test="paired_t")

print(f"Mean Splade:   {comp.mean_a:.4f}")
print(f"Mean Baseline: {comp.mean_b:.4f}")
print(f"Difference:    {comp.mean_diff:+.4f}")
print(f"t-statistic:   {comp.statistic:.4f}")
print(f"p-value:       {comp.pvalue:.4f}")
print(f"Significant:   {comp.significant(0.05)}")
```

### Supported Statistical Tests

| Test Name (`test=`) | Description |
| :--- | :--- |
| `'paired_t'` (default) | **Paired Student's t-test**: Parametric test for paired differences. Exact two-tailed p-value computed via the regularized incomplete beta function. |
| `'permutation'` | **Randomized Sign-Flip Permutation Test**: Non-parametric Fisher randomization test drawing $B$ Monte Carlo sign-flip resamples ($s_i \in \{-1, +1\}$). Uses unbiased $(k+1)/(B+1)$ p-value estimator. |
| `'bootstrap'` | **Studentized Bootstrap Test**: Non-parametric test drawing $B$ resamples with replacement from null-centered differences. |

For randomized tests (`permutation`, `bootstrap`), specify `num_resamples` (default `10000`) and an optional integer `seed` for deterministic reproducibility:

```python
comp_perm = res_splade.compare(
    res_baseline,
    measure="map",
    test="permutation",
    num_resamples=10000,
    seed=42,
)
```

---

## 2. Multiple Testing Corrections (`adjust_pvalues`)

Evaluating $m$ hypotheses increases the risk of false discoveries (Type I errors). `trec_eval.adjust_pvalues()` provides standard zero-dependency multiple comparison adjustment methods:

```python
from trec_eval import adjust_pvalues

raw_pvalues = [0.005, 0.032, 0.041, 0.180]

# Holm-Bonferroni step-down (default FWER control)
p_holm = adjust_pvalues(raw_pvalues, method="holm")
# [0.020, 0.082, 0.082, 0.180]

# Benjamini-Hochberg (FDR control)
p_fdr = adjust_pvalues(raw_pvalues, method="fdr_bh")

# Bonferroni (single-step alpha / m)
p_bonf = adjust_pvalues(raw_pvalues, method="bonferroni")
```

### Correction Methods

- **`'holm'`** (default): Holm-Bonferroni step-down method (Holm, 1979). Strongly controls Family-Wise Error Rate (FWER $\le \alpha$) with higher statistical power than single-step Bonferroni without assuming hypothesis independence. This is useful when comparing a number of runs against a baseline (see an example below).
- **`'fdr_bh'`**: Benjamini-Hochberg procedure (Benjamini & Hochberg, 1995). Controls False Discovery Rate (FDR); optimal for large multi-model benchmark screening. This is useful when making a number of pairwise significance tests (see example below) when commonly you might use the Bonferroni correction. The FDR correction is less conservative.
- **`'bonferroni'`**: Single-step Bonferroni multiplier ($p_i \cdot m$). Conservative FWER control.
- **`'none'`**: Returns unadjusted raw p-values.

---

## 3. Comparing Candidates Against a Baseline (`compare_against_baseline`)

When benchmarking multiple candidate runs against a single control baseline, use `evaluator.compare_against_baseline()`:

```python
evaluator = trec_eval.Evaluator("qrels.txt", measures=["map", "ndcg@10"])

comp_table = evaluator.compare_against_baseline(
    baseline="runs/bm25.txt",
    candidates={
        "splade": "runs/splade.txt",
        "colbert": "runs/colbert.txt",
        "cross_encoder": "runs/rerank.txt",
    },
    measures=["map", "ndcg@10"],
    test="paired_t",
    correction="holm",
    alpha=0.05,
)

# Print formatted summary table
print(comp_table)

# Export to Pandas DataFrame (requires 'pandas' or 'polars')
df = comp_table.to_dataframe()
# Columns: ['candidate', 'measure', 'baseline_score', 'candidate_score', 'diff', 'statistic', 'p_raw', 'p_adj', 'significant']
```

---

## 4. All-Pairs System Comparison Matrix (`compare_all`)

To compute full pairwise significance matrices across multiple systems:

```python
matrix = evaluator.compare_all(
    runs={
        "bm25": "runs/bm25.txt",
        "splade": "runs/splade.txt",
        "colbert": "runs/colbert.txt",
    },
    measure="map",
    test="paired_t",
    correction="fdr_bh",
    alpha=0.05,
)

# Print pairwise matrix with significance markers (▲ / ▼)
print(matrix.summary_table())

# Access adjusted p-values table (requires 'pandas')
df_pvalues = matrix.to_dataframe()
```

