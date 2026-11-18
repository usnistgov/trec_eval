//! A repeated document is refused only where trec_eval refuses it: in a topic that is scored,
//! within `-M`, when a requested measure reads the ranking. In two-topics.qrels, topic 1 judges
//! d1 (1), d2 (0) and d3 (2), and topic 2 judges d1 (0) and d2 (1). The expected values are
//! trec_eval's for the same files.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/scored-duplicates")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(args: &[&str], qrels: &str, run: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .args([fixture(qrels), fixture(run)])
        .output()
        .expect("failed to run te-rust")
}

fn summary(output: &Output, measure: &str) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 3 && fields[0] == measure && fields[1] == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))[2]
        .to_string()
}

/// Asserts exit status 4, trec_eval's status for a repeat, and returns stderr.
fn refusal(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(4), "{output:?}");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_repeat_in_a_run_topic_absent_from_the_qrels_is_not_checked() {
    // Topic 9 is never scored. Topic 1 ranks d1, d2, d3: AP = (1/1 + 2/3) / 2 = 0.8333.
    // Topic 2 ranks d2 first: AP = 1. map = 0.9167.
    let out = te_rust(&["-m", "map"], "two-topics.qrels", "repeat-in-topic-9.run");
    assert_eq!(summary(&out, "map"), "0.9167");
}

#[test]
fn a_repeat_beyond_max_docs_is_not_checked() {
    // The second d2 is at rank 3. With -M 2, topic 1 ranks d1, d2: AP = (1/1) / 2 = 0.5, and
    // map = (0.5 + 1) / 2 = 0.7500.
    let out = te_rust(
        &["-M", "2", "-m", "map"],
        "two-topics.qrels",
        "repeat-at-rank-3.run",
    );
    assert_eq!(summary(&out, "map"), "0.7500");
    let out = te_rust(
        &["-R", "qrels_jg", "-M", "2", "-m", "map_avgjg"],
        "two-topics.qrels",
        "repeat-at-rank-3.run",
    );
    assert_eq!(summary(&out, "map_avgjg"), "0.7500");
}

#[test]
fn a_repeat_within_max_docs_is_refused() {
    let stderr = refusal(&te_rust(
        &["-m", "map"],
        "two-topics.qrels",
        "repeat-at-rank-3.run",
    ));
    assert!(
        stderr.contains("repeat-at-rank-3.run': duplicate document 'd2' for query '1'"),
        "{stderr}"
    );
    let out = te_rust(
        &["-R", "qrels_jg", "-m", "map_avgjg"],
        "two-topics.qrels",
        "repeat-at-rank-3.run",
    );
    assert!(refusal(&out).contains("duplicate document 'd2' for query '1'"));
}

#[test]
fn measures_that_read_no_ranking_skip_the_check() {
    // num_q and runid form no ranking in trec_eval, which prints num_q 2 for both files.
    let out = te_rust(&["-m", "num_q"], "two-topics.qrels", "repeat-at-rank-3.run");
    assert_eq!(summary(&out, "num_q"), "2");
    let out = te_rust(
        &["-m", "num_q"],
        "repeat-in-topic-1.qrels",
        "two-topics.run",
    );
    assert_eq!(summary(&out, "num_q"), "2");
    let out = te_rust(
        &["-m", "runid", "-m", "num_q"],
        "two-topics.qrels",
        "repeat-at-rank-3.run",
    );
    assert_eq!(summary(&out, "runid"), "r");
}

#[test]
fn a_repeated_judgment_is_refused_only_in_a_scored_topic() {
    let stderr = refusal(&te_rust(
        &["-m", "map"],
        "repeat-in-topic-1.qrels",
        "two-topics.run",
    ));
    assert!(
        stderr.contains("repeat-in-topic-1.qrels': duplicate document 'd1' for query '1'"),
        "{stderr}"
    );
    // Topic 3 has no results. Without -c it is not scored, and map is that of topics 1 and 2.
    // trec_eval 10.x exits 4 here because topic 3 has no results, as docs/cli.md says.
    let out = te_rust(&["-m", "map"], "repeat-in-topic-3.qrels", "two-topics.run");
    assert_eq!(summary(&out, "map"), "0.9167");
    // With -c topic 3 is scored, and trec_eval refuses the repeat too.
    let out = te_rust(
        &["-c", "-m", "map"],
        "repeat-in-topic-3.qrels",
        "two-topics.run",
    );
    assert!(refusal(&out).contains("duplicate document 'x' for query '3'"));
}
