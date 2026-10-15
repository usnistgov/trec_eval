//! Multi-run statistical significance testing and hypothesis tests for IR evaluation.
//!
//! # Primary Mathematical & Algorithmic References
//! - **Student's t-Distribution & Two-Tailed Tail Probability**:
//!   Abramowitz, M., & Stegun, I. A. (1964). *Handbook of Mathematical Functions with Formulas,
//!   Graphs, and Mathematical Tables*. National Bureau of Standards Applied Mathematics Series 55.
//!   - Section 26.7.1, Eq. 26.7.1: Two-tailed probability $P(|T| \ge |t|) = I_{\frac{\nu}{\nu + t^2}}\left(\frac{\nu}{2}, \frac{1}{2}\right)$.
//! - **Regularized Incomplete Beta Function Continued Fraction (Modified Lentz Method)**:
//!   Press, W. H., Teukolsky, S. A., Vetterling, W. T., & Flannery, B. P. (2007). *Numerical Recipes:
//!   The Art of Scientific Computing* (3rd ed.). Cambridge University Press.
//!   - Section 6.4 "Incomplete Beta Function", Eq. 6.4.2 & 6.4.5.
//!   - Section 5.2 "Evaluation of Continued Fractions" (Modified Lentz's method).
//! - **Log-Gamma Function via Lanczos Approximation**:
//!   Lanczos, C. (1964). "A precision approximation of the gamma function." *SIAM Journal on
//!   Numerical Analysis, Series B*, 1: 86–96.
//!   - Press et al. (2007), *Numerical Recipes* (3rd ed.), §6.1, Eq. 6.1.5 ($g = 7, N = 9$).
//! - **Paired Samples t-Test**:
//!   Snedecor, G. W., & Cochran, W. G. (1989). *Statistical Methods* (8th ed.). Iowa State University
//!   Press, §6.3 "Paired Samples".
//! - **Monte Carlo Permutation P-Values**:
//!   Phipson, B., & Smyth, G. K. (2010). "Permutation P-values should never be zero: calculating exact
//!   P-values when permutations are randomly drawn." *Statistical Applications in Genetics and
//!   Molecular Biology*, 9(1): Article 39. <https://doi.org/10.2202/1544-6115.1585>
//! - **Significance Testing in IR Benchmarking**:
//!   Smucker, M. D., Allan, J., & Carterette, B. (2007). "A comparison of statistical significance
//!   tests for information retrieval evaluation." *Proceedings of the 16th ACM Conference on Information
//!   and Knowledge Management (CIKM '07)*, pp. 623–632. <https://doi.org/10.1145/1321440.1321528>


use crate::eval::bootstrap::SplitMix64Rng;

/// Result of a paired statistical significance test between two retrieval systems across topic queries.
#[derive(Debug, Clone, PartialEq)]
pub struct SignificanceTestResult {
    /// Name of the test performed ("paired_t", "permutation", or "bootstrap").
    pub test_name: String,
    /// Test statistic (e.g., Student's t value for t-test; observed mean difference for permutation/bootstrap).
    pub statistic: f64,
    /// Two-tailed p-value testing the null hypothesis $H_0: \mu_A - \mu_B = 0$.
    pub pvalue: f64,
    /// Sample mean of system A across topics ($\bar{x}_A$).
    pub mean_a: f64,
    /// Sample mean of system B across topics ($\bar{x}_B$).
    pub mean_b: f64,
    /// Sample mean difference ($\bar{d} = \bar{x}_A - \bar{x}_B$).
    pub mean_diff: f64,
    /// Number of paired topics evaluated ($n$).
    pub n: usize,
}

/// Compute natural log of the Gamma function $\ln \Gamma(z)$ for $z > 0$.
///
/// Implements the 9-coefficient Lanczos approximation with $g = 7$, achieving machine precision
/// ($|\text{error}| < 2 \times 10^{-15}$) across the positive real line.
///
/// Reference: Press et al. (2007), *Numerical Recipes* (3rd ed.), §6.1, Eq. 6.1.3–6.1.5.
pub fn ln_gamma(z: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    // Lanczos coefficients for g = 7, N = 9 (Numerical Recipes §6.1)
    const LANCZOS_COEFFICIENTS: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.138571095856205,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    let mut sum = LANCZOS_COEFFICIENTS[0];
    let z_adjusted = z - 1.0;
    for (idx, &coeff) in LANCZOS_COEFFICIENTS.iter().enumerate().skip(1) {
        sum += coeff / (z_adjusted + idx as f64);
    }

    let base = z_adjusted + 7.5; // (z - 0.5 + g) where g = 7
    let sqrt_two_pi = (2.0 * std::f64::consts::PI).sqrt();
    let ln_sqrt_two_pi = sqrt_two_pi.ln();

    ln_sqrt_two_pi + (z_adjusted + 0.5) * base.ln() - base + sum.ln()
}

/// Regularized Incomplete Beta Function $I_x(a, b) = \frac{B(x; a, b)}{B(a, b)}$ for $x \in [0, 1], a > 0, b > 0$.
///
/// Evaluated via modified Lentz's method on the continued fraction expansion (Numerical Recipes §6.4.2).
/// When $x > \frac{a + 1}{a + b + 2}$, uses the symmetry relation $I_x(a, b) = 1 - I_{1-x}(b, a)$
/// to guarantee rapid convergence.
///
/// Reference: Press et al. (2007), *Numerical Recipes* (3rd ed.), §6.4, Eq. 6.4.2–6.4.5.
pub fn inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Symmetry relation to accelerate continued fraction convergence (Numerical Recipes §6.4.3)
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - inc_beta(b, a, 1.0 - x);
    }

    // Prefactor: exp(a * ln(x) + b * ln(1 - x) - ln(Beta(a, b))) / a
    let ln_beta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let prefactor = (a * x.ln() + b * (1.0 - x).ln() - ln_beta).exp() / a;

    // Continued fraction evaluation via modified Lentz's method (Numerical Recipes §5.2)
    const MAX_ITERATIONS: usize = 200;
    const CONVERGENCE_EPSILON: f64 = 1e-15;
    const FLOAT_TINY: f64 = 1e-30;

    let mut fraction_estimate = 1.0;
    let mut c_ratio = 1.0;
    let mut d_ratio = 0.0;

    for iteration in 1..=MAX_ITERATIONS {
        let m = iteration as f64;

        // Even step numerator: d_{2m} (Eq. 6.4.5)
        let numerator_even = -(a + m - 1.0) * (a + b + m - 1.0) * x
            / ((a + 2.0 * m - 2.0) * (a + 2.0 * m - 1.0));

        d_ratio = 1.0 + numerator_even * d_ratio;
        if d_ratio.abs() < FLOAT_TINY {
            d_ratio = FLOAT_TINY;
        }
        c_ratio = 1.0 + numerator_even / c_ratio;
        if c_ratio.abs() < FLOAT_TINY {
            c_ratio = FLOAT_TINY;
        }
        d_ratio = 1.0 / d_ratio;
        fraction_estimate *= c_ratio * d_ratio;

        // Odd step numerator: d_{2m+1} (Eq. 6.4.5)
        let numerator_odd = m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m));

        d_ratio = 1.0 + numerator_odd * d_ratio;
        if d_ratio.abs() < FLOAT_TINY {
            d_ratio = FLOAT_TINY;
        }
        c_ratio = 1.0 + numerator_odd / c_ratio;
        if c_ratio.abs() < FLOAT_TINY {
            c_ratio = FLOAT_TINY;
        }
        d_ratio = 1.0 / d_ratio;

        let delta = c_ratio * d_ratio;
        fraction_estimate *= delta;

        if (delta - 1.0).abs() < CONVERGENCE_EPSILON {
            break;
        }
    }

    // Eq. 6.4.2: I_x(a, b) = prefactor / (1 + CF)
    prefactor / fraction_estimate
}


/// Calculate the exact two-tailed p-value for Student's $t$ statistic with $\nu$ degrees of freedom.
///
/// Uses the exact closed-form relation to the regularized incomplete beta function:
/// $$P(|T| \ge |t|) = I_{\frac{\nu}{\nu + t^2}}\left(\frac{\nu}{2}, \frac{1}{2}\right)$$
///
/// Reference: Abramowitz & Stegun (1964), *Handbook of Mathematical Functions*, §26.7.1.
pub fn student_t_two_tailed_pvalue(t_statistic: f64, degrees_of_freedom: f64) -> f64 {
    if degrees_of_freedom <= 0.0 {
        return 1.0;
    }
    if !t_statistic.is_finite() {
        return 0.0;
    }
    let beta_x = degrees_of_freedom / (degrees_of_freedom + t_statistic * t_statistic);
    inc_beta(0.5 * degrees_of_freedom, 0.5, beta_x)
}

/// Paired Student's t-test comparing aligned topic score vectors $x_A$ and $x_B$.
///
/// Tests the null hypothesis that the mean paired difference is zero ($\mu_D = 0$), where
/// $D_i = x_{A, i} - x_{B, i}$.
///
/// $$t = \frac{\bar{D}}{\sqrt{s_D^2 / n}}, \quad \text{where } s_D^2 = \frac{1}{n-1} \sum_{i=1}^n (D_i - \bar{D})^2$$
///
/// References:
/// - Snedecor, G. W., & Cochran, W. G. (1989). *Statistical Methods* (8th ed.), §6.3 "Paired Samples".
/// - Abramowitz, M., & Stegun, I. A. (1964). *Handbook of Mathematical Functions*, §26.7.1 (p-value via incomplete beta).
pub fn paired_t_test(system_a_scores: &[f64], system_b_scores: &[f64]) -> SignificanceTestResult {
    let sample_size = std::cmp::min(system_a_scores.len(), system_b_scores.len());
    if sample_size == 0 {
        return SignificanceTestResult {
            test_name: "paired_t".to_string(),
            statistic: 0.0,
            pvalue: 1.0,
            mean_a: 0.0,
            mean_b: 0.0,
            mean_diff: 0.0,
            n: 0,
        };
    }

    let mean_a = system_a_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let mean_b = system_b_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let mean_diff = mean_a - mean_b;

    if sample_size < 2 {
        return SignificanceTestResult {
            test_name: "paired_t".to_string(),
            statistic: if mean_diff == 0.0 { 0.0 } else { f64::NAN },
            pvalue: if mean_diff == 0.0 { 1.0 } else { 0.0 },
            mean_a,
            mean_b,
            mean_diff,
            n: sample_size,
        };
    }

    let paired_differences: Vec<f64> = (0..sample_size)
        .map(|i| system_a_scores[i] - system_b_scores[i])
        .collect();

    let sample_variance: f64 = paired_differences
        .iter()
        .map(|&diff| (diff - mean_diff).powi(2))
        .sum::<f64>()
        / (sample_size - 1) as f64;

    let (t_statistic, pvalue) = if sample_variance <= 1e-15 {
        if mean_diff.abs() <= 1e-15 {
            (0.0, 1.0)
        } else {
            (
                if mean_diff > 0.0 { f64::INFINITY } else { f64::NEG_INFINITY },
                0.0,
            )
        }
    } else {
        let standard_error = (sample_variance / sample_size as f64).sqrt();
        let t = mean_diff / standard_error;
        let df = (sample_size - 1) as f64;
        let p = student_t_two_tailed_pvalue(t, df);
        (t, p)
    };

    SignificanceTestResult {
        test_name: "paired_t".to_string(),
        statistic: t_statistic,
        pvalue: pvalue.clamp(0.0, 1.0),
        mean_a,
        mean_b,
        mean_diff,
        n: sample_size,
    }
}

/// Randomized sign-flip permutation test for paired topic score differences.
///
/// Under the null hypothesis $H_0$ that system A and system B perform identically, the sign of each
/// paired topic difference $D_i = x_{A, i} - x_{B, i}$ is equally likely to be positive or negative
/// (Fisher's randomization principle).
///
/// For $B$ Monte Carlo resamples, signs $s_i \in \{-1, +1\}$ are drawn uniformly at random to compute
/// pseudo-mean differences $\bar{D}^{(b)} = \frac{1}{n}\sum_{i=1}^n s_i D_i$.
/// The two-tailed p-value is estimated as:
///
/// $$p = \frac{\sum_{b=1}^B \mathbb{I}\left(|\bar{D}^{(b)}| \ge |\bar{D}_{\text{obs}}|\right) + 1}{B + 1}$$
///
/// Reference:
/// - Smucker, Allan, & Carterette (CIKM 2007), §3.2 "Randomization Test".
/// - Phipson & Smyth (2010), "Permutation P-values should never be zero", Eq. 1.
pub fn permutation_test(
    system_a_scores: &[f64],
    system_b_scores: &[f64],
    num_resamples: usize,
    seed: Option<u64>,
) -> SignificanceTestResult {
    let sample_size = std::cmp::min(system_a_scores.len(), system_b_scores.len());
    if sample_size == 0 {
        return SignificanceTestResult {
            test_name: "permutation".to_string(),
            statistic: 0.0,
            pvalue: 1.0,
            mean_a: 0.0,
            mean_b: 0.0,
            mean_diff: 0.0,
            n: 0,
        };
    }

    let mean_a = system_a_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let mean_b = system_b_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let observed_mean_diff = mean_a - mean_b;

    if observed_mean_diff.abs() <= 1e-15 {
        return SignificanceTestResult {
            test_name: "permutation".to_string(),
            statistic: 0.0,
            pvalue: 1.0,
            mean_a,
            mean_b,
            mean_diff: 0.0,
            n: sample_size,
        };
    }

    let paired_differences: Vec<f64> = (0..sample_size)
        .map(|i| system_a_scores[i] - system_b_scores[i])
        .collect();
    let observed_abs_diff = observed_mean_diff.abs();

    let mut rng = match seed {
        Some(s) => SplitMix64Rng::new(s),
        None => SplitMix64Rng::default(),
    };

    let total_resamples = if num_resamples == 0 { 10_000 } else { num_resamples };
    let mut extreme_count = 0;

    for _ in 0..total_resamples {
        let mut resample_sum = 0.0;
        for &diff in &paired_differences {
            // Flip sign with probability 0.5 under null hypothesis
            let sign = if (rng.next_u64() & 1) == 1 { 1.0 } else { -1.0 };
            resample_sum += sign * diff;
        }
        let resample_mean = resample_sum / sample_size as f64;
        if resample_mean.abs() >= observed_abs_diff - 1e-15 {
            extreme_count += 1;
        }
    }

    // Unbiased Monte Carlo p-value estimator (Phipson & Smyth, 2010)
    let pvalue = (extreme_count as f64 + 1.0) / (total_resamples as f64 + 1.0);

    SignificanceTestResult {
        test_name: "permutation".to_string(),
        statistic: observed_mean_diff,
        pvalue: pvalue.clamp(0.0, 1.0),
        mean_a,
        mean_b,
        mean_diff: observed_mean_diff,
        n: sample_size,
    }
}

/// Studentized Bootstrap test of mean difference between paired topic score vectors.
///
/// Tests $H_0: \mu_D = 0$ by centering the differences $Z_i = D_i - \bar{D}$ so their mean is exactly 0,
/// drawing $B$ bootstrap resamples of size $n$ with replacement from $\{Z_i\}$, and computing
/// the proportion of resample means exceeding the observed difference in magnitude.
///
/// Reference:
/// - Efron & Tibshirani (1993), *An Introduction to the Bootstrap*, §16.2 "Hypothesis Testing with the Bootstrap".
/// - Smucker, Allan, & Carterette (CIKM 2007), §3.3 "The Bootstrap".
pub fn bootstrap_test(
    system_a_scores: &[f64],
    system_b_scores: &[f64],
    num_resamples: usize,
    seed: Option<u64>,
) -> SignificanceTestResult {
    let sample_size = std::cmp::min(system_a_scores.len(), system_b_scores.len());
    if sample_size == 0 {
        return SignificanceTestResult {
            test_name: "bootstrap".to_string(),
            statistic: 0.0,
            pvalue: 1.0,
            mean_a: 0.0,
            mean_b: 0.0,
            mean_diff: 0.0,
            n: 0,
        };
    }

    let mean_a = system_a_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let mean_b = system_b_scores[..sample_size].iter().sum::<f64>() / sample_size as f64;
    let observed_mean_diff = mean_a - mean_b;

    if observed_mean_diff.abs() <= 1e-15 {
        return SignificanceTestResult {
            test_name: "bootstrap".to_string(),
            statistic: 0.0,
            pvalue: 1.0,
            mean_a,
            mean_b,
            mean_diff: 0.0,
            n: sample_size,
        };
    }

    let paired_differences: Vec<f64> = (0..sample_size)
        .map(|i| system_a_scores[i] - system_b_scores[i])
        .collect();

    // Center differences to enforce the null hypothesis expectation E[Z] = 0
    let centered_differences: Vec<f64> = paired_differences
        .iter()
        .map(|&diff| diff - observed_mean_diff)
        .collect();
    let observed_abs_diff = observed_mean_diff.abs();

    let mut rng = match seed {
        Some(s) => SplitMix64Rng::new(s),
        None => SplitMix64Rng::default(),
    };

    let total_resamples = if num_resamples == 0 { 10_000 } else { num_resamples };
    let mut extreme_count = 0;

    for _ in 0..total_resamples {
        let mut resample_sum = 0.0;
        for _ in 0..sample_size {
            let random_idx = rng.gen_range(sample_size);
            resample_sum += centered_differences[random_idx];
        }
        let resample_mean = resample_sum / sample_size as f64;
        if resample_mean.abs() >= observed_abs_diff - 1e-15 {
            extreme_count += 1;
        }
    }

    let pvalue = (extreme_count as f64 + 1.0) / (total_resamples as f64 + 1.0);

    SignificanceTestResult {
        test_name: "bootstrap".to_string(),
        statistic: observed_mean_diff,
        pvalue: pvalue.clamp(0.0, 1.0),
        mean_a,
        mean_b,
        mean_diff: observed_mean_diff,
        n: sample_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_student_t_distribution_known_values() {
        // Critical values from standard statistical tables (Abramowitz & Stegun Table 26.1):
        // df = 10, t = 2.228139 -> two-tailed p ≈ 0.05
        let p_df10 = student_t_two_tailed_pvalue(2.2281388519649385, 10.0);
        assert!((p_df10 - 0.05).abs() < 1e-5, "Expected ~0.05 for df=10, got {}", p_df10);

        // df = 30, t = 2.042272 -> two-tailed p ≈ 0.05
        let p_df30 = student_t_two_tailed_pvalue(2.0422724563012373, 30.0);
        assert!((p_df30 - 0.05).abs() < 1e-5, "Expected ~0.05 for df=30, got {}", p_df30);

        // df = 50, t = 2.677793 -> two-tailed p ≈ 0.01
        let p_df50_01 = student_t_two_tailed_pvalue(2.677793272445832, 50.0);
        assert!((p_df50_01 - 0.01).abs() < 1e-5, "Expected ~0.01 for df=50, got {}", p_df50_01);

        // t = 0 -> p = 1.0
        assert_eq!(student_t_two_tailed_pvalue(0.0, 10.0), 1.0);
    }

    #[test]
    fn test_paired_t_identical_vectors() {
        let scores_a = vec![0.5, 0.4, 0.8, 0.2];
        let scores_b = vec![0.5, 0.4, 0.8, 0.2];
        let res = paired_t_test(&scores_a, &scores_b);
        assert_eq!(res.statistic, 0.0);
        assert_eq!(res.pvalue, 1.0);
        assert_eq!(res.mean_diff, 0.0);
        assert_eq!(res.n, 4);
    }

    #[test]
    fn test_paired_t_significant_difference() {
        let scores_a = vec![0.8, 0.85, 0.9, 0.82, 0.88];
        let scores_b = vec![0.2, 0.25, 0.3, 0.22, 0.28];
        let res = paired_t_test(&scores_a, &scores_b);
        assert!(res.statistic > 20.0);
        assert!(res.pvalue < 1e-4);
        assert!((res.mean_diff - 0.6).abs() < 1e-6);
        assert_eq!(res.n, 5);
    }

    #[test]
    fn test_permutation_test_deterministic_reproducibility() {
        let scores_a = vec![0.8, 0.85, 0.9, 0.82, 0.88, 0.79, 0.84, 0.87, 0.83, 0.86];
        let scores_b = vec![0.2, 0.25, 0.3, 0.22, 0.28, 0.21, 0.24, 0.27, 0.23, 0.26];
        let res1 = permutation_test(&scores_a, &scores_b, 1000, Some(42));
        let res2 = permutation_test(&scores_a, &scores_b, 1000, Some(42));
        assert_eq!(res1.pvalue, res2.pvalue);
        assert!(res1.pvalue < 0.05);
    }


    #[test]
    fn test_bootstrap_test_deterministic_reproducibility() {
        let scores_a = vec![0.8, 0.85, 0.9, 0.82, 0.88];
        let scores_b = vec![0.2, 0.25, 0.3, 0.22, 0.28];
        let res1 = bootstrap_test(&scores_a, &scores_b, 1000, Some(42));
        let res2 = bootstrap_test(&scores_a, &scores_b, 1000, Some(42));
        assert_eq!(res1.pvalue, res2.pvalue);
        assert!(res1.pvalue < 0.05);
    }
}
