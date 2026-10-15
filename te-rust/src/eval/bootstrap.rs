//! Bootstrap resampling and confidence interval estimation (ISSUES #22).
//!
//! Provides non-parametric percentile bootstrap confidence intervals for evaluation
//! summary metrics. Designed as a self-contained, reproducible core component
//! reusable by both the CLI binary and the future Python bindings.

/// Minimal, fast SplitMix64 PRNG for deterministic, reproducible bootstrap resampling.
#[derive(Debug, Clone)]
pub struct SplitMix64Rng {
    state: u64,
}

impl SplitMix64Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate next pseudo-random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    /// Generate an index uniformly in 0..bound.
    pub fn gen_range(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() % (bound as u64)) as usize
    }
}

impl Default for SplitMix64Rng {
    fn default() -> Self {
        // Default seed derived from standard constant
        Self::new(0x853c49e6748fea9b)
    }
}

/// Configuration for bootstrap confidence interval estimation.
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Number of bootstrap resamples (typically 1000 or 2000).
    pub num_samples: usize,
    /// Significance level alpha (e.g. 0.05 for 95% CI).
    pub alpha: f64,
    /// Optional seed for deterministic RNG.
    pub seed: Option<u64>,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            num_samples: 1000,
            alpha: 0.05,
            seed: None,
        }
    }
}

/// A computed confidence interval for a metric summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceInterval {
    /// The original sample point estimate (e.g. sample mean).
    pub point_estimate: f64,
    /// Lower bound of the confidence interval (alpha/2 percentile).
    pub lower: f64,
    /// Upper bound of the confidence interval (1 - alpha/2 percentile).
    pub upper: f64,
}

/// Compute percentile bootstrap confidence intervals for the arithmetic mean
/// of an array of scores.
pub fn bootstrap_mean_ci(scores: &[f64], config: &BootstrapConfig) -> Option<ConfidenceInterval> {
    if scores.is_empty() {
        return None;
    }

    let mean = scores.iter().sum::<f64>() / (scores.len() as f64);
    if scores.len() == 1 {
        return Some(ConfidenceInterval {
            point_estimate: mean,
            lower: mean,
            upper: mean,
        });
    }

    bootstrap_ci_with_aggregator(scores, config, |sample| {
        sample.iter().sum::<f64>() / (sample.len() as f64)
    })
}

/// Compute percentile bootstrap confidence intervals with a custom aggregation function.
pub fn bootstrap_ci_with_aggregator<F>(
    scores: &[f64],
    config: &BootstrapConfig,
    aggregate: F,
) -> Option<ConfidenceInterval>
where
    F: Fn(&[f64]) -> f64,
{
    let n = scores.len();
    if n == 0 {
        return None;
    }

    let point_estimate = aggregate(scores);
    if n == 1 || config.num_samples == 0 {
        return Some(ConfidenceInterval {
            point_estimate,
            lower: point_estimate,
            upper: point_estimate,
        });
    }

    let mut rng = match config.seed {
        Some(s) => SplitMix64Rng::new(s),
        None => {
            // Mix time-based entropy for default non-deterministic sampling
            let time_entropy = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x853c49e6748fea9b);
            SplitMix64Rng::new(time_entropy)
        }
    };

    let b = config.num_samples;
    let mut replicates = Vec::with_capacity(b);
    let mut sample_buf = vec![0.0; n];

    for _ in 0..b {
        for val in sample_buf.iter_mut() {
            let idx = rng.gen_range(n);
            *val = scores[idx];
        }
        let stat = aggregate(&sample_buf);
        replicates.push(stat);
    }

    // Sort replicates using total_cmp for determinism
    replicates.sort_by(|a, b| a.total_cmp(b));

    // Calculate percentiles: alpha/2 and 1 - alpha/2
    let alpha = config.alpha.clamp(0.0001, 0.9999);
    let lower_idx = ((alpha / 2.0) * (b as f64)).floor() as usize;
    let upper_idx = (((1.0 - alpha / 2.0) * (b as f64)).ceil() as usize).min(b - 1);

    let lower = replicates[lower_idx.min(b - 1)];
    let upper = replicates[upper_idx.min(b - 1)];

    Some(ConfidenceInterval {
        point_estimate,
        lower,
        upper,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_scores() {
        let config = BootstrapConfig::default();
        assert_eq!(bootstrap_mean_ci(&[], &config), None);
    }

    #[test]
    fn test_single_score() {
        let config = BootstrapConfig::default();
        let ci = bootstrap_mean_ci(&[0.5], &config).unwrap();
        assert_eq!(
            ci,
            ConfidenceInterval {
                point_estimate: 0.5,
                lower: 0.5,
                upper: 0.5,
            }
        );
    }

    #[test]
    fn test_constant_scores() {
        let config = BootstrapConfig {
            num_samples: 500,
            alpha: 0.05,
            seed: Some(42),
        };
        let scores = vec![0.75; 50];
        let ci = bootstrap_mean_ci(&scores, &config).unwrap();
        assert_eq!(ci.point_estimate, 0.75);
        assert!((ci.lower - 0.75).abs() < 1e-9);
        assert!((ci.upper - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_deterministic_seeding() {
        let config1 = BootstrapConfig {
            num_samples: 1000,
            alpha: 0.05,
            seed: Some(12345),
        };
        let config2 = BootstrapConfig {
            num_samples: 1000,
            alpha: 0.05,
            seed: Some(12345),
        };
        let scores = vec![0.1, 0.2, 0.4, 0.5, 0.8, 0.9];
        let ci1 = bootstrap_mean_ci(&scores, &config1).unwrap();
        let ci2 = bootstrap_mean_ci(&scores, &config2).unwrap();
        assert_eq!(ci1, ci2);
        assert!(ci1.lower <= ci1.point_estimate);
        assert!(ci1.point_estimate <= ci1.upper);
    }

    #[test]
    fn test_custom_aggregator_geometric_mean() {
        let config = BootstrapConfig {
            num_samples: 500,
            alpha: 0.05,
            seed: Some(999),
        };
        let scores = vec![0.1, 0.4, 0.9];
        let ci = bootstrap_ci_with_aggregator(&scores, &config, |s| {
            let log_sum: f64 = s.iter().map(|&x: &f64| x.ln()).sum();
            (log_sum / s.len() as f64).exp()
        })
        .unwrap();

        let expected_geo_mean = (0.1 * 0.4 * 0.9_f64).powf(1.0 / 3.0);
        assert!((ci.point_estimate - expected_geo_mean).abs() < 1e-6);
        assert!(ci.lower <= ci.point_estimate);
        assert!(ci.point_estimate <= ci.upper);
    }
}
