use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gm-complete-floor")
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

fn test_data(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/test-data")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn missing_topic_counts_once_under_complete() {
    // AP and bpref are 1 on topic 1 and 0.5 on topic 2. Topic 3 has no results.
    // exp((ln 1 + ln 0.5 + ln 1e-5) / 3) = 0.0171
    let out = te_rust(&[
        "-c",
        "-m",
        "gm_map",
        "-m",
        "gm_bpref",
        &fixture("three-topics.qrels"),
        &fixture("no-topic-3.run"),
    ]);
    assert_eq!(summary(&out, "gm_map"), "0.0171");
    assert_eq!(summary(&out, "gm_bpref"), "0.0171");
}

#[test]
fn missing_topic_scores_like_a_topic_with_nothing_relevant() {
    // With -c, a missing topic must equal a topic whose results are all unjudged.
    let qrels = fixture("three-topics.qrels");
    let missing = te_rust(&["-c", "-m", "gm_map", &qrels, &fixture("no-topic-3.run")]);
    let unjudged = te_rust(&["-c", "-m", "gm_map", &qrels, &fixture("all-topics.run")]);
    assert_eq!(summary(&missing, "gm_map"), summary(&unjudged, "gm_map"));
}

#[test]
fn without_complete_a_full_run_is_unchanged() {
    // Topic 3 retrieves only an unjudged document, so its AP is 0 and it scores the floor.
    // exp((ln 1 + ln 0.5 + ln 1e-5) / 3) = 0.0171
    let out = te_rust(&[
        "-m",
        "gm_map",
        &fixture("three-topics.qrels"),
        &fixture("all-topics.run"),
    ]);
    assert_eq!(summary(&out, "gm_map"), "0.0171");
}

#[test]
fn matches_trec_eval_reference_output() {
    // out.test.aqc is trec_eval's expected output for `-q -c -m all_trec`.
    let out = te_rust(&[
        "-q",
        "-c",
        "-m",
        "all_trec",
        &test_data("qrels.test"),
        &test_data("results.trunc"),
    ]);
    let reference = std::fs::read_to_string(test_data("out.test.aqc")).unwrap();
    for measure in ["gm_map", "gm_bpref"] {
        let expected = reference
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .find(|fields| fields[0] == measure && fields[1] == "all")
            .unwrap()[2]
            .to_string();
        assert_eq!(summary(&out, measure), expected, "{measure}");
    }
}
