use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rbp-unjudged-gain")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .output()
        .expect("failed to run te-rust")
}

/// Printed rows as (measure, topic, value), in output order.
fn rows(output: &Output) -> Vec<(String, String, String)> {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let mut fields = line.split_whitespace();
            let mut next = || fields.next().unwrap_or_default().to_string();
            (next(), next(), next())
        })
        .collect()
}

/// The printed value of `measure` on the `all` row.
fn summary(output: &Output, measure: &str) -> String {
    rows(output)
        .into_iter()
        .find(|(name, topic, _)| name == measure && topic == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))
        .2
}

#[test]
fn unjudged_document_keeps_gain_zero_when_a_gain_is_negative() {
    // Gains 0 -> -1, 1 -> 1 normalise to 0 and 1. x at rank 1 is unjudged and gains 0.
    // a at rank 2: 0.1 * 0.9 = 0.0900
    let out = te_rust(&[
        "--global-gains",
        "0=-1,1=1",
        "-m",
        "rbp",
        &fixture("binary.qrels"),
        &fixture("unjudged-first.run"),
    ]);
    assert_eq!(summary(&out, "rbp"), "0.0900");
}

#[test]
fn judged_documents_normalise_by_the_table_range() {
    // Gains -2, 0, 6 for c, b, a normalise to 0, 0.25, 1. u at rank 2 is unjudged and gains 0.
    // 0.1 * (0 + 0 + 1 * 0.81 + 0.25 * 0.729) = 0.099225
    let out = te_rust(&[
        "--global-gains",
        "0=-2,1=0,2=6",
        "-m",
        "rbp",
        &fixture("three-grades.qrels"),
        &fixture("three-grades.run"),
    ]);
    assert_eq!(summary(&out, "rbp"), "0.0992");
}

#[test]
fn default_gains_are_unchanged() {
    // 0.1 * (1 + 2/3 * 0.81 + 1/3 * 0.6561) = 0.1759
    let out = te_rust(&[
        "-m",
        "rbp",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp"), "0.1759");
}
