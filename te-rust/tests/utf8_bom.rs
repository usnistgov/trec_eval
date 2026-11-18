//! A UTF-8 byte-order mark at the start of a file. The files with the mark are written at test
//! time from the committed fixtures.

use std::path::PathBuf;
use std::process::{Command, Output};

const BOM: &[u8] = b"\xEF\xBB\xBF";

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/utf8-bom")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// The fixture `name` with a BOM before line `line` (1-based), in Cargo's temporary directory.
fn with_bom(name: &str, line: usize) -> String {
    let text = std::fs::read_to_string(fixture(name)).unwrap();
    let mut bytes = Vec::new();
    for (i, l) in text.lines().enumerate() {
        if i + 1 == line {
            bytes.extend_from_slice(BOM);
        }
        bytes.extend_from_slice(l.as_bytes());
        bytes.push(b'\n');
    }
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("bom-line{line}-{name}"));
    std::fs::write(&path, bytes).unwrap();
    path.to_string_lossy().into_owned()
}

fn te_rust(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .output()
        .expect("failed to run te-rust")
}

/// The `measure topic value` rows of stdout, with the spacing removed.
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
fn a_bom_before_the_first_run_line_is_skipped() {
    // Before, the first line had qid "\u{feff}1", a topic the qrels lack, so topic 1 kept two of
    // its three documents: num_ret 2.
    let run = with_bom("two-topics.run", 1);
    let out = te_rust(&["-q", "-m", "num_ret", &fixture("two-topics.qrels"), &run]);
    assert_eq!(rows(&out), ["num_ret 1 3", "num_ret 2 2", "num_ret all 5"]);
}

#[test]
fn a_bom_before_the_first_qrels_line_is_skipped() {
    // Before, topic 1 lost its judgment of d1 (num_rel 1 instead of 2), and -c counted the
    // phantom topic "\u{feff}1": num_q 3.
    let qrels = with_bom("two-topics.qrels", 1);
    let out = te_rust(&["-q", "-m", "num_rel", &qrels, &fixture("two-topics.run")]);
    assert_eq!(rows(&out), ["num_rel 1 2", "num_rel 2 1", "num_rel all 3"]);
    let out = te_rust(&["-c", "-m", "num_q", &qrels, &fixture("two-topics.run")]);
    assert_eq!(rows(&out), ["num_q all 2"]);
}

#[test]
fn a_bom_after_the_first_line_is_still_data() {
    // Only the start of the file is checked: the mark before line 4 makes the qid "\u{feff}2",
    // so topic 2 has one document left in the run.
    let run = with_bom("two-topics.run", 4);
    let out = te_rust(&["-q", "-m", "num_ret", &fixture("two-topics.qrels"), &run]);
    assert_eq!(rows(&out), ["num_ret 1 3", "num_ret 2 1", "num_ret all 4"]);
}
