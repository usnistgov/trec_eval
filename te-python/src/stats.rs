use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use te_rust::eval::significance::{
    bootstrap_test, paired_t_test, permutation_test, SignificanceTestResult,
};

/// Result of a statistical significance test between two evaluation runs.
#[pyclass(name = "ComparisonResult")]
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    #[pyo3(get)]
    pub test_name: String,
    #[pyo3(get)]
    pub measure: String,
    #[pyo3(get)]
    pub statistic: f64,
    #[pyo3(get)]
    pub pvalue: f64,
    #[pyo3(get)]
    pub mean_a: f64,
    #[pyo3(get)]
    pub mean_b: f64,
    #[pyo3(get)]
    pub mean_diff: f64,
    #[pyo3(get)]
    pub n: usize,
}

#[pymethods]
impl ComparisonResult {
    /// Check whether the observed difference is statistically significant at significance level alpha.
    #[pyo3(signature = (alpha = 0.05))]
    pub fn significant(&self, alpha: f64) -> bool {
        self.pvalue < alpha
    }

    fn __repr__(&self) -> String {
        let sig_marker = if self.pvalue < 0.001 {
            "***"
        } else if self.pvalue < 0.01 {
            "**"
        } else if self.pvalue < 0.05 {
            "*"
        } else {
            "ns"
        };
        format!(
            "<ComparisonResult: {} on '{}', mean_A={:.4}, mean_B={:.4}, diff={:+.4}, stat={:.4}, p={:.4} ({})>",
            self.test_name, self.measure, self.mean_a, self.mean_b, self.mean_diff, self.statistic, self.pvalue, sig_marker
        )
    }
}

/// Compute a paired significance test between two aligned score vectors in Rust.
pub fn run_significance_test(
    scores_a: &[f64],
    scores_b: &[f64],
    measure: &str,
    test: &str,
    num_resamples: usize,
    seed: Option<u64>,
) -> PyResult<ComparisonResult> {
    let test_lower = test.to_ascii_lowercase();
    let result: SignificanceTestResult = match test_lower.as_str() {
        "paired_t" | "t_test" | "ttest" | "t" => paired_t_test(scores_a, scores_b),
        "permutation" | "randomization" | "perm" => {
            permutation_test(scores_a, scores_b, num_resamples, seed)
        }
        "bootstrap" | "boot" => bootstrap_test(scores_a, scores_b, num_resamples, seed),
        _ => {
            return Err(PyValueError::new_err(format!(
                "Unknown statistical test '{}'. Supported tests: 'paired_t', 'permutation', 'bootstrap'.",
                test
            )));
        }
    };

    Ok(ComparisonResult {
        test_name: result.test_name,
        measure: measure.to_string(),
        statistic: result.statistic,
        pvalue: result.pvalue,
        mean_a: result.mean_a,
        mean_b: result.mean_b,
        mean_diff: result.mean_diff,
        n: result.n,
    })
}
