use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/complete-set-summary")
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
fn complete_set_prints_the_average_over_qrels_topics() {
    let out = te_rust(&[
        "-c",
        "-m",
        "map",
        "-m",
        "num_q",
        "-m",
        "gm_map",
        &fixture("topics-1-2.qrels"),
        &fixture("topic-3.run"),
    ]);
    assert_eq!(summary(&out, "num_q"), "2");
    assert_eq!(summary(&out, "map"), "0.0000");
    // The floor of 1e-5 rounds to 0.0000 at four decimals.
    assert_eq!(summary(&out, "gm_map"), "0.0000");
}

#[test]
fn qrels_jg_prints_the_average_too() {
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-c",
        "-m",
        "map_avgjg",
        &fixture("topics-1-2.qrels_jg"),
        &fixture("topic-3.run"),
    ]);
    assert_eq!(summary(&out, "map_avgjg"), "0.0000");
}

#[test]
fn confidence_intervals_cover_the_complete_set() {
    let out = te_rust(&[
        "-c",
        "-C",
        "--seed",
        "1",
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topic-3.run"),
    ]);
    for row in ["map", "map_ci_lower", "map_ci_upper"] {
        assert_eq!(summary(&out, row), "0.0000", "{row}");
    }
}

#[test]
fn no_summary_flag_is_still_respected() {
    let out = te_rust(&[
        "-c",
        "-n",
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topic-3.run"),
    ]);
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn overlapping_topics_are_unchanged() {
    // Topic 1: (1 + 2/3) / 2 at -l 1 with d1 and d3 relevant at ranks 1, 3. Topic 2: 1.
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topics-1-2.run"),
    ]);
    assert_eq!(summary(&out, "map"), "0.9167");
}
