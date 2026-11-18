use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/grade-prefix")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// A copy of a file from the C tree's `test/` directory.
fn test_data(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/test-data")
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

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn summary(output: &Output, measure: &str) -> String {
    stdout(output)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 3 && fields[0] == measure && fields[1] == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))[2]
        .to_string()
}

#[test]
fn fractional_grades_read_as_their_leading_integer() {
    // a is graded 1.5 and b 0.9. Read as 1 and 0, only a is relevant, and a is at rank 2:
    // AP = 1/2. Rounding would make b relevant too and give AP = 1.
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("grades.qrels"),
        &fixture("grades.run"),
    ]);
    assert_eq!(summary(&out, "map"), "0.5000");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Line 1: value '1.5' partially read as '1'"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Line 2: value '0.9' partially read as '0'"),
        "{stderr}"
    );
}

#[test]
fn qrels_jg_grades_follow_the_same_rule() {
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-m",
        "map_avgjg",
        &fixture("grades.qrels"),
        &fixture("grades.run"),
    ]);
    assert_eq!(summary(&out, "map_avgjg"), "0.5000");
    assert!(String::from_utf8_lossy(&out.stderr).contains("partially read as '0'"));
}

#[test]
fn integer_grades_print_no_warning() {
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("integer.qrels"),
        &fixture("grades.run"),
    ]);
    assert_eq!(summary(&out, "map"), "0.5000");
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn grade_without_a_leading_integer_is_refused() {
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("no-digits.qrels"),
        &fixture("grades.run"),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid integer for field 'rel': '.5'"));
}

#[test]
fn with_floats_file_scores_like_qrels_test() {
    // The Makefile check: `trec_eval test/qrels.test.with-floats test/results.test` must print
    // `test/out.test`, the output for `test/qrels.test`.
    let results = test_data("results.test");
    let floats = te_rust(&["-q", &test_data("qrels.test.with-floats"), &results]);
    let integers = te_rust(&["-q", &test_data("qrels.test"), &results]);
    assert_eq!(stdout(&floats), stdout(&integers));
}
