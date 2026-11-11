use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/exit-status")
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

fn status(args: &[&str]) -> i32 {
    te_rust(args)
        .status
        .code()
        .expect("te-rust ended on a signal")
}

fn eval_status(measure: &str, qrels: &str, run: &str) -> i32 {
    status(&["-m", measure, &fixture(qrels), &fixture(run)])
}

// Each expected status is what the C tree in trec_eval/ and v10.0 (f425365) return on these files.
#[test]
fn input_failures_exit_2() {
    assert_eq!(eval_status("map", "absent.qrels", "good.run"), 2);
    assert_eq!(eval_status("map", "good.qrels", "absent.run"), 2);
    assert_eq!(eval_status("map", "malformed.qrels", "good.run"), 2);
    assert_eq!(eval_status("map", "good.qrels", "malformed.run"), 2);
    assert_eq!(eval_status("map", "empty.qrels", "good.run"), 2);
    assert_eq!(eval_status("map", "good.qrels", "empty.run"), 2);
    assert_eq!(
        status(&["-R", "nosuch", &fixture("good.qrels"), &fixture("good.run")]),
        2
    );
    let jg = |qrels: &str| {
        status(&[
            "-R",
            "qrels_jg",
            "-m",
            "map_avgjg",
            &fixture(qrels),
            &fixture("good.run"),
        ])
    };
    assert_eq!(jg("malformed.qrels"), 2);
    assert_eq!(jg("duplicate-doc.qrels"), 2);
}

#[test]
fn unusable_measure_parameters_exit_2() {
    assert_eq!(eval_status("P.x", "good.qrels", "good.run"), 2);
    assert_eq!(eval_status("map_cut.x", "good.qrels", "good.run"), 2);
}

#[test]
fn illegal_measure_names_exit_1() {
    assert_eq!(eval_status("nosuch", "good.qrels", "good.run"), 1);
    assert_eq!(eval_status("P.", "good.qrels", "good.run"), 1);
}

#[test]
fn unscorable_topics_exit_4() {
    assert_eq!(eval_status("map", "duplicate-doc.qrels", "good.run"), 4);
    assert_eq!(eval_status("map", "good.qrels", "duplicate-doc.run"), 4);
    assert_eq!(eval_status("map", "good.qrels", "other-topic.run"), 4);
}

#[test]
fn usage_errors_exit_1() {
    assert_eq!(status(&["-m", "map", &fixture("good.qrels")]), 1);
    assert_eq!(
        status(&["--nosuchflag", &fixture("good.qrels"), &fixture("good.run")]),
        1
    );
}

#[test]
fn success_and_help_exit_0() {
    assert_eq!(eval_status("map", "good.qrels", "good.run"), 0);
    assert_eq!(status(&["--help"]), 0);
    assert_eq!(status(&["--version"]), 0);
}
