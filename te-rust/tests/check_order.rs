use std::path::PathBuf;
use std::process::{Command, Output};

// good.* hold one valid topic, malformed.* a line with too few fields, and
// duplicate-doc.* the document d1 twice in topic 1. absent.* do not exist.
fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/check-order")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(measures: &[&str], qrels: &str, run: &str) -> Output {
    let mut args: Vec<String> = Vec::new();
    for measure in measures {
        args.push("-m".to_string());
        args.push(measure.to_string());
    }
    args.push(fixture(qrels));
    args.push(fixture(run));
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(&args)
        .output()
        .expect("failed to run te-rust")
}

/// The exit status, and whether stderr names `file`.
fn reported(output: &Output, file: &str) -> (i32, bool) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    (
        output.status.code().expect("te-rust ended on a signal"),
        stderr.contains(file),
    )
}

// Each expected status is what the C tree in trec_eval/ and v10.0 (f425365)
// return. trec_eval refuses an illegal measure name while it reads its
// options (status 1), then reads the qrels and the run (2), checks the
// measure parameters (2), and finds a repeated document while it scores (4).
#[test]
fn illegal_measure_name_is_reported_before_the_files() {
    let status = |measures: &[&str], qrels, run| te_rust(measures, qrels, run).status.code();
    assert_eq!(status(&["nosuch"], "good.qrels", "absent.run"), Some(1));
    assert_eq!(
        status(&["nosuch"], "malformed.qrels", "duplicate-doc.run"),
        Some(1)
    );
    // A name is checked before any parameter, whatever their order.
    assert_eq!(
        status(&["P.x", "nosuch"], "good.qrels", "good.run"),
        Some(1)
    );
    assert_eq!(status(&["P.x", "P."], "good.qrels", "good.run"), Some(1));
}

#[test]
fn qrels_problem_is_reported_before_the_run() {
    let output = te_rust(&["map"], "absent.qrels", "absent.run");
    assert_eq!(reported(&output, "absent.qrels"), (2, true));
    let output = te_rust(&["map"], "malformed.qrels", "duplicate-doc.run");
    assert_eq!(reported(&output, "malformed.qrels"), (2, true));
}

#[test]
fn judgment_format_is_checked_before_the_run() {
    let output = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args([
            "-R",
            "nosuch",
            &fixture("good.qrels"),
            &fixture("duplicate-doc.run"),
        ])
        .output()
        .expect("failed to run te-rust");
    assert_eq!(reported(&output, "nosuch"), (2, true));
}

#[test]
fn repeated_qrels_document_is_reported_after_the_run() {
    let output = te_rust(&["map"], "duplicate-doc.qrels", "malformed.run");
    assert_eq!(reported(&output, "malformed.run"), (2, true));
    let output = te_rust(&["map"], "duplicate-doc.qrels", "good.run");
    assert_eq!(reported(&output, "duplicate-doc.qrels"), (4, true));
}

#[test]
fn unusable_parameter_with_a_file_problem_exits_2() {
    // trec_eval reports the file and te-rust the parameter, with the same status.
    assert_eq!(
        te_rust(&["P.x"], "absent.qrels", "good.run").status.code(),
        Some(2)
    );
    assert_eq!(
        te_rust(&["P.x"], "good.qrels", "absent.run").status.code(),
        Some(2)
    );
    assert_eq!(
        te_rust(&["P.x"], "duplicate-doc.qrels", "good.run")
            .status
            .code(),
        Some(2)
    );
}
