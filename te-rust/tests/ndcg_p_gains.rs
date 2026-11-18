//! The ndcg_p gains and discount that docs/measures/gains.md describes.

use std::path::PathBuf;
use std::process::Command;

// topic.qrels grades d1 1, d2 0 and d3 2, and topic.run ranks d1, d2, d3.
fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ndcg-p-gains")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// The printed `all` value of `measure`.
fn summary(measure: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args([
            "-m",
            measure,
            &fixture("topic.qrels"),
            &fixture("topic.run"),
        ])
        .output()
        .expect("failed to run te-rust");
    assert!(output.status.success(), "{measure}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .nth(2)
        .unwrap_or_else(|| panic!("no value for {measure}: {stdout}"))
        .to_string()
}

#[test]
fn ndcg_p_uses_level_gains_and_the_original_discount() {
    // Rank 1 is not discounted and rank i >= 2 is divided by log2(i):
    // DCG = 1 + 0/1 + 2/log2(3) = 2.26186, ideal 2, 1, 0 gives 2 + 1/1 = 3.
    assert_eq!(summary("ndcg_p"), "0.7540");
    // Exponential gains 2^r - 1 would give (1 + 3/log2(3)) / (3 + 1) = 0.7232.
    // ndcg divides rank i by log2(i + 1): (1 + 2/2) / (2 + 1/log2(3)) = 0.7602.
    assert_eq!(summary("ndcg"), "0.7602");
}

#[test]
fn ndcg_p_takes_custom_gains() {
    // (3.5 + 9/log2(3)) / (9 + 3.5/1) = 9.17837 / 12.5.
    assert_eq!(summary("ndcg_p.1=3.5,2=9.0"), "0.7343");
    // Level 0 may have a gain, and a gain may be negative. The ideal order is
    // d3 (2), d2 (1), and stops at d1 (-1): (-1 + 1/1 + 2/log2(3)) / (2 + 1/1).
    assert_eq!(summary("ndcg_p.0=1,1=-1"), "0.4206");
}
