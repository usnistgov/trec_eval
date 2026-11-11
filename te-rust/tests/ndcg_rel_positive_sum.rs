use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ndcg-rel-positive-sum")
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
fn negative_sum_reports_zero() {
    // f has gain -1 at rank 1, e has gain 1 at rank 2. DCG at e is -1 + 1 / log2(3) < 0.
    let out = te_rust(&[
        "-m",
        "ndcg_rel.0=-1,1=1",
        &fixture("topic.qrels"),
        &fixture("topic.run"),
    ]);
    assert_eq!(summary(&out, "ndcg_rel_0=-1,1=1"), "0.0000");
}

#[test]
fn positive_sum_is_unchanged() {
    // e at rank 2: (1 / log2(3)) / 1 = 0.6309
    let out = te_rust(&[
        "-m",
        "ndcg_rel",
        &fixture("topic.qrels"),
        &fixture("topic.run"),
    ]);
    assert_eq!(summary(&out, "ndcg_rel"), "0.6309");
}
