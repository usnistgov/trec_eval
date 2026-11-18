//! `dcg` and `ideal_dcg`, which trec_eval 10.0 registers beside `ndcg` (`m_ndcg.c`).
//!
//! The fixture has two topics. Topic 1 judges d1 at grade 1, d2 at 2 and d3 at 0, and the
//! run ranks d1, x (unjudged), d2, d3. Topic 2 judges d1 at 0 and d2 at 1, and the run ranks
//! d1, d2. `three-topics.qrels` adds topic 3, which judges d1 at 2 and which the run lacks.
//! Every expected value below is a hand calculation, and trec_eval 10.0 prints the same
//! rows, apart from the requests that crash it (#195).

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dcg-measures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn eval_with(qrels: &str, extra: &[&str]) -> Output {
    let (qrels, run) = (fixture(qrels), fixture("two-topics.run"));
    let mut args = extra.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(&args)
        .output()
        .expect("failed to run te-rust")
}

fn eval(extra: &[&str]) -> Output {
    eval_with("two-topics.qrels", extra)
}

/// The printed rows, as `measure topic value`, in order.
fn rows(output: &Output) -> Vec<String> {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

#[test]
fn dcg_and_ideal_dcg_print_the_two_sums_of_ndcg() {
    // Topic 1: dcg = 1/log2(2) + 2/log2(4) = 2; the ideal order 2, 1 gives
    // ideal_dcg = 2 + 1/log2(3) = 2.6309, and ndcg = 2 / 2.6309 = 0.7602.
    // Topic 2: dcg = 1/log2(3) = 0.6309, ideal_dcg = 1, ndcg = 0.6309.
    // Each summary is the mean of its own topic values: (2 + 0.6309) / 2 = 1.3155,
    // (2.6309 + 1) / 2 = 1.8155 and (0.7602 + 0.6309) / 2 = 0.6956. The mean of ndcg is
    // not the ratio of the means.
    assert_eq!(
        rows(&eval(&["-q", "-m", "dcg", "-m", "ideal_dcg", "-m", "ndcg"])),
        [
            "dcg 1 2.0000",
            "ideal_dcg 1 2.6309",
            "ndcg 1 0.7602",
            "dcg 2 0.6309",
            "ideal_dcg 2 1.0000",
            "ndcg 2 0.6309",
            "dcg all 1.3155",
            "ideal_dcg all 1.8155",
            "ndcg all 0.6956",
        ]
    );
}

#[test]
fn gain_map_sets_the_gains_and_the_row_name() {
    // Gain 3 for grade 1. Topic 1: dcg = 3 + 2/log2(4) = 4, ideal_dcg = 3 + 2/log2(3) =
    // 4.2619. Topic 2: dcg = 3/log2(3) = 1.8928, ideal_dcg = 3.
    assert_eq!(
        rows(&eval(&["-q", "-m", "dcg.1=3"])),
        ["dcg_1=3 1 4.0000", "dcg_1=3 2 1.8928", "dcg_1=3 all 2.9464"]
    );
    assert_eq!(
        rows(&eval(&["-q", "-m", "ideal_dcg.1=3"])),
        [
            "ideal_dcg_1=3 1 4.2619",
            "ideal_dcg_1=3 2 3.0000",
            "ideal_dcg_1=3 all 3.6309",
        ]
    );
}

#[test]
fn gains_that_are_not_positive_count_in_dcg_only() {
    // Gain -1 for grade 0. dcg adds it at every rank: topic 1 gains -1/log2(5) at d3, so
    // 2 - 0.4307 = 1.5693, and topic 2 gains -1 at d1, so -1 + 0.6309 = -0.3691.
    // ideal_dcg stops at the first gain that is not positive and is unchanged.
    assert_eq!(
        rows(&eval(&["-q", "-m", "dcg.0=-1", "-m", "ideal_dcg.0=-1"])),
        [
            "dcg_0=-1 1 1.5693",
            "ideal_dcg_0=-1 1 2.6309",
            "dcg_0=-1 2 -0.3691",
            "ideal_dcg_0=-1 2 1.0000",
            "dcg_0=-1 all 0.6001",
            "ideal_dcg_0=-1 all 1.8155",
        ]
    );
    // With gain 0 for grade 1 as well, topic 2 has no positive gain: ideal_dcg is 0, and
    // dcg still sums the ranking, -1 at d1.
    assert_eq!(
        rows(&eval(&[
            "-q",
            "-m",
            "dcg.0=-1,1=0",
            "-m",
            "ideal_dcg.0=-1,1=0"
        ])),
        [
            "dcg_0=-1,1=0 1 0.5693",
            "ideal_dcg_0=-1,1=0 1 2.0000",
            "dcg_0=-1,1=0 2 -1.0000",
            "ideal_dcg_0=-1,1=0 2 0.0000",
            "dcg_0=-1,1=0 all -0.2153",
            "ideal_dcg_0=-1,1=0 all 1.0000",
        ]
    );
}

#[test]
fn each_measure_reads_only_its_own_gain_map() {
    // Each row is the value that the measure prints alone. trec_eval 10.0 shares one
    // parameter list among dcg, ideal_dcg and ndcg, and crashes on these requests (#195).
    assert_eq!(
        rows(&eval(&["-m", "dcg.1=3", "-m", "ideal_dcg", "-m", "ndcg"])),
        [
            "dcg_1=3 all 2.9464",
            "ideal_dcg all 1.8155",
            "ndcg all 0.6956"
        ]
    );
    // ndcg.1=3: topic 1 is 4 / 4.2619 = 0.9386 and topic 2 is 1.8928 / 3 = 0.6309.
    assert_eq!(
        rows(&eval(&["-m", "dcg", "-m", "ndcg.1=3"])),
        ["dcg all 1.3155", "ndcg_1=3 all 0.7847"]
    );
}

#[test]
fn global_gains_apply_unless_the_measure_has_a_gain_map() {
    assert_eq!(
        rows(&eval(&[
            "--global-gains",
            "1=3",
            "-m",
            "dcg",
            "-m",
            "ideal_dcg"
        ])),
        ["dcg all 2.9464", "ideal_dcg all 3.6309"]
    );
    // dcg.2=5 keeps gain 1 for grade 1: topic 1 is 1 + 5/log2(4) = 3.5 and topic 2 is
    // 0.6309, as `-m dcg.2=5` alone.
    assert_eq!(
        rows(&eval(&["--global-gains", "1=3", "-m", "dcg.2=5"])),
        ["dcg_2=5 all 2.0655"]
    );
}

#[test]
fn complete_set_scores_a_missing_topic_as_an_empty_ranking() {
    // Under -c, topic 3 has dcg 0 and ideal_dcg 2, the gain of its one judged document at
    // rank 1. The means are over 3 topics: 2.6309 / 3 = 0.8770 and 5.6309 / 3 = 1.8770.
    assert_eq!(
        rows(&eval_with(
            "three-topics.qrels",
            &["-c", "-q", "-m", "dcg", "-m", "ideal_dcg"]
        )),
        [
            "dcg 1 2.0000",
            "ideal_dcg 1 2.6309",
            "dcg 2 0.6309",
            "ideal_dcg 2 1.0000",
            "dcg 3 0.0000",
            "ideal_dcg 3 2.0000",
            "dcg all 0.8770",
            "ideal_dcg all 1.8770",
        ]
    );
}

#[test]
fn unusable_gain_maps_are_refused_as_for_ndcg() {
    for (measure, message) in [
        ("dcg.1=x", "invalid gain 'x' for relevance level 1"),
        ("ideal_dcg.x=1", "invalid relevance level 'x'"),
        ("dcg.1=2,1=5", "relevance level 1 is given more than once"),
        ("ideal_dcg.5,5", "expected key=value, got '5'"),
    ] {
        let out = eval(&["-m", measure]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{measure} was accepted");
        assert!(out.stdout.is_empty(), "{measure} printed output");
        assert!(stderr.contains(message), "{measure}: {stderr}");
    }
}
