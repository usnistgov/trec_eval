"""
Statistical significance testing and multiple testing corrections for IR evaluation.

References:
- Holm, S. (1979). "A simple sequentially rejective multiple test procedure."
  Scandinavian Journal of Statistics, 6(2): 65–70.
- Benjamini, Y., & Hochberg, Y. (1995). "Controlling the false discovery rate:
  a practical and powerful approach to multiple testing."
  Journal of the Royal Statistical Society: Series B, 57(1): 289–300.
- Bonferroni, C. E. (1936). "Teoria statistica delle classi e calcolo delle probabilità."
  Pubblicazioni del R Istituto Superiore di Scienze Economiche e Commerciali di Firenze, 8: 3–62.
- Smucker, M. D., Allan, J., & Carterette, B. (2007). "A comparison of statistical
  significance tests for information retrieval evaluation." CIKM '07, pp. 623–632.
"""

from typing import Any, Dict, List, NamedTuple, Optional, Sequence


class ComparisonRow(NamedTuple):
    candidate: str
    measure: str
    baseline_score: float
    candidate_score: float
    diff: float
    statistic: float
    p_raw: float
    p_adj: float
    significant: bool


def adjust_pvalues(pvalues: Sequence[float], method: str = "holm") -> List[float]:
    """Adjust p-values for multiple comparisons hypothesis testing.

    Parameters
    ----------
    pvalues : Sequence[float]
        Sequence of unadjusted raw p-values.
    method : str, default 'holm'
        Correction method:
        - 'holm': Holm-Bonferroni step-down (FWER control <= alpha)
        - 'fdr_bh': Benjamini-Hochberg (FDR control)
        - 'bonferroni': Single-step Bonferroni (alpha / m)
        - 'none': No adjustment (returns raw p-values)

    Returns
    -------
    List[float]
        Adjusted p-values in the original input ordering.
    """
    m = len(pvalues)
    if m <= 1 or method.lower() == "none":
        return [float(p) for p in pvalues]

    method_lower = method.lower()

    if method_lower == "bonferroni":
        return [min(1.0, max(0.0, float(p) * m)) for p in pvalues]

    # Sort p-values ascending while keeping track of original indices
    indexed_pvalues = sorted(enumerate(pvalues), key=lambda x: x[1])

    if method_lower in ("holm", "holm_bonferroni"):
        # Holm (1979) step-down method
        adjusted_sorted = []
        running_max = 0.0
        for rank, (_, p) in enumerate(indexed_pvalues, start=1):
            multiplier = m - rank + 1
            step_val = min(1.0, max(0.0, float(p) * multiplier))
            running_max = max(running_max, step_val)
            adjusted_sorted.append(running_max)

    elif method_lower in ("fdr_bh", "bh", "benjamini_hochberg"):
        # Benjamini-Hochberg (1995) step-up method
        raw_sorted = [x[1] for x in indexed_pvalues]
        step_vals = [min(1.0, max(0.0, float(p) * m / (rank + 1))) for rank, p in enumerate(raw_sorted)]
        # Enforce reverse monotonicity
        adjusted_sorted = [0.0] * m
        running_min = 1.0
        for i in reversed(range(m)):
            running_min = min(running_min, step_vals[i])
            adjusted_sorted[i] = running_min

    else:
        raise ValueError(
            f"Unknown correction method '{method}'. Supported methods: 'holm', 'fdr_bh', 'bonferroni', 'none'."
        )

    # Reconstruct original ordering
    final_pvalues = [0.0] * m
    for (orig_idx, _), adj_p in zip(indexed_pvalues, adjusted_sorted):
        final_pvalues[orig_idx] = adj_p

    return final_pvalues


class MultiComparisonTable:
    """Summary table of multi-candidate comparisons against a baseline system."""

    def __init__(self, rows: List[ComparisonRow], alpha: float, test: str, correction: str):
        self.rows = rows
        self.alpha = alpha
        self.test = test
        self.correction = correction

    def to_dict(self) -> List[Dict[str, Any]]:
        return [row._asdict() for row in self.rows]

    def to_dataframe(self) -> Any:
        try:
            import pandas as pd
            return pd.DataFrame(self.to_dict())
        except ImportError:
            try:
                import polars as pl
                return pl.DataFrame(self.to_dict())
            except ImportError:
                raise ImportError(
                    "pandas or polars is required for .to_dataframe(). Install via 'pip install trec-eval[pandas]'."
                )

    def __len__(self) -> int:
        return len(self.rows)

    def __iter__(self):
        return iter(self.rows)

    def __repr__(self) -> str:
        headers = ["Candidate", "Measure", "Baseline", "Candidate", "Diff", "p-raw", f"p-adj ({self.correction})", "Sig"]
        col_fmt = "{:<16} {:<12} {:>9} {:>9} {:>10} {:>9} {:>16} {:>5}"

        lines = [
            f"Multi-run Comparison (test: {self.test}, alpha: {self.alpha}):",
            "-" * 85,
            col_fmt.format(*headers),
            "-" * 85,
        ]
        for r in self.rows:
            sig_marker = "★" if r.significant else "ns"
            lines.append(col_fmt.format(
                r.candidate,
                r.measure,
                f"{r.baseline_score:.4f}",
                f"{r.candidate_score:.4f}",
                f"{r.diff:+.4f}",
                f"{r.p_raw:.4f}",
                f"{r.p_adj:.4f}",
                sig_marker,
            ))
        return "\n".join(lines)


class AllPairsMatrix:
    """Pairwise comparison matrix across multiple systems."""

    def __init__(
        self,
        systems: List[str],
        measure: str,
        test: str,
        correction: str,
        alpha: float,
        pvalues_raw: Dict[str, Dict[str, float]],
        pvalues_adj: Dict[str, Dict[str, float]],
        mean_diffs: Dict[str, Dict[str, float]],
    ):
        self.systems = systems
        self.measure = measure
        self.test = test
        self.correction = correction
        self.alpha = alpha
        self.pvalues_raw = pvalues_raw
        self.pvalues_adjusted = pvalues_adj
        self.mean_diffs = mean_diffs

    def to_dataframe(self) -> Any:
        try:
            import pandas as pd
            return pd.DataFrame(self.pvalues_adjusted, index=self.systems)
        except ImportError:
            raise ImportError("pandas is required for .to_dataframe(). Install via 'pip install trec-eval[pandas]'.")

    def summary_table(self) -> str:
        header = ["System"] + self.systems
        col_w = max(12, max(len(s) for s in self.systems) + 2)
        fmt = "{:<" + str(col_w) + "}" + "".join(["{:>" + str(col_w) + "}" for _ in self.systems])
        lines = [
            f"Pairwise Comparisons Matrix (measure: {self.measure}, test: {self.test}, adj: {self.correction}):",
            "-" * (col_w * (len(self.systems) + 1)),
            fmt.format(*header),
            "-" * (col_w * (len(self.systems) + 1)),
        ]
        for s1 in self.systems:
            row_vals = [s1]
            for s2 in self.systems:
                if s1 == s2:
                    row_vals.append("-")
                else:
                    padj = self.pvalues_adjusted[s1][s2]
                    diff = self.mean_diffs[s1][s2]
                    marker = "▲" if (padj < self.alpha and diff > 0) else ("▼" if (padj < self.alpha and diff < 0) else "")
                    row_vals.append(f"{padj:.4f}{marker}")
            lines.append(fmt.format(*row_vals))
        return "\n".join(lines)

    def __repr__(self) -> str:
        return self.summary_table()
