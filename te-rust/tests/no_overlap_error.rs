use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/no-overlap-error")
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
fn no_shared_topic_is_an_error() {
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topic-3.run"),
    ]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no topic has both results and judgments")
    );
}

#[test]
fn qrels_jg_follows_the_same_rule() {
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-m",
        "map_avgjg",
        &fixture("topics-1-2.qrels_jg"),
        &fixture("topic-3.run"),
    ]);
    assert!(!out.status.success());
}

#[test]
fn complete_set_is_not_an_error() {
    let out = te_rust(&[
        "-c",
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topic-3.run"),
    ]);
    assert!(out.status.success());
}

#[test]
fn overlapping_topics_are_unchanged() {
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("topics-1-2.qrels"),
        &fixture("topics-1-2.run"),
    ]);
    assert_eq!(summary(&out, "map"), "0.9167");
}
