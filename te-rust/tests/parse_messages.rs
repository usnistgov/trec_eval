use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/parse-messages")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// Runs te-rust on two fixtures, asserts that it fails, and returns stderr.
fn refusal(flags: &[&str], qrels: &str, run: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(flags)
        .args([fixture(qrels), fixture(run)])
        .output()
        .expect("failed to run te-rust");
    assert!(
        !out.status.success(),
        "te-rust accepted {qrels} and {run}: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn qrels_with_only_comments_has_no_data_lines() {
    // trec_eval with #124: "No data lines in qrels file".
    let stderr = refusal(&[], "comments.qrels", "one.run");
    assert!(stderr.contains("No data lines"), "{stderr}");
    assert!(!stderr.contains("File is empty"), "{stderr}");
}

#[test]
fn qrels_jg_with_only_comments_has_no_data_lines() {
    let stderr = refusal(&["-R", "qrels_jg"], "comments.qrels", "one.run");
    assert!(stderr.contains("No data lines"), "{stderr}");
}

#[test]
fn run_with_only_comments_and_blank_lines_has_no_data_lines() {
    // trec_eval with #117: "No data lines in results file".
    let stderr = refusal(&[], "one.qrels", "comments.run");
    assert!(stderr.contains("No data lines"), "{stderr}");
}

#[test]
fn a_file_without_lines_is_still_empty() {
    let stderr = refusal(&[], "one.qrels", "empty.run");
    assert!(stderr.contains("File is empty"), "{stderr}");
}

#[test]
fn qrels_lines_need_exactly_four_fields() {
    for (flags, qrels, expected) in [
        (
            &[][..],
            "five-fields.qrels",
            "Line 1: expected 4 fields, found 5",
        ),
        (
            &[][..],
            "three-fields.qrels",
            "Line 1: expected 4 fields, found 3",
        ),
        (
            &["-R", "qrels_jg"][..],
            "five-fields.qrels",
            "Line 1: expected 4 fields, found 5",
        ),
    ] {
        let stderr = refusal(flags, qrels, "one.run");
        assert!(stderr.contains(expected), "{stderr}");
    }
}

#[test]
fn run_lines_need_at_least_six_fields() {
    let stderr = refusal(&[], "one.qrels", "five-fields.run");
    assert!(
        stderr.contains("Line 1: expected at least 6 fields, found 5"),
        "{stderr}"
    );
}
