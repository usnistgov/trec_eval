use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/blank-qrels-lines")
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

#[test]
fn blank_qrels_lines_are_skipped_with_a_warning() {
    // Line 2 is empty and line 4 holds only spaces. trec_eval stops at line 2 with
    // "Malformed line 2" and exits 2.
    let out = te_rust(&["-m", "map", &fixture("blank.qrels"), &fixture("blank.run")]);
    assert_eq!(summary(&out, "map"), "0.5000");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Line 2: blank line skipped"), "{stderr}");
    assert!(stderr.contains("Line 4: blank line skipped"), "{stderr}");
}

#[test]
fn blank_qrels_jg_lines_are_skipped_with_a_warning() {
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-m",
        "map_avgjg",
        &fixture("blank.qrels"),
        &fixture("blank.run"),
    ]);
    assert_eq!(summary(&out, "map_avgjg"), "0.5000");
    assert!(String::from_utf8_lossy(&out.stderr).contains("Line 2: blank line skipped"));
}

#[test]
fn blank_run_lines_are_skipped_silently() {
    // trec_eval's run reader also skips blank lines without a message.
    let out = te_rust(&["-m", "map", &fixture("plain.qrels"), &fixture("blank.run")]);
    assert_eq!(summary(&out, "map"), "0.5000");
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
