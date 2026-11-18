//! Relevance levels from -127 to 127, the range that `get_qrels.c:43` documents and trec_eval
//! enforces with #123. The fixtures are those of trec_eval's `test_grade-range.sh`.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/grade-range")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(args: &[&str], qrels: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .args([fixture(qrels), fixture("grade.run")])
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
fn levels_127_and_minus_127_are_read() {
    // d1 is relevant at rank 1 in both topics (level 1, then level 127): map 1.0000, as in
    // trec_eval.
    assert_eq!(
        summary(&te_rust(&["-m", "map"], "edge.qrels"), "map"),
        "1.0000"
    );
    let out = te_rust(&["-R", "qrels_jg", "-m", "map_avgjg"], "edge.qrels");
    assert_eq!(summary(&out, "map_avgjg"), "1.0000");
}

#[test]
fn levels_outside_the_range_are_refused() {
    // Before, te-rust accepted levels up to 1,000,000 in size: 128.qrels gave map 1.0000.
    for (qrels, level) in [("128.qrels", "128"), ("minus-128.qrels", "-128")] {
        for args in [
            &["-m", "map"][..],
            &["-R", "qrels_jg", "-m", "map_avgjg"][..],
        ] {
            let out = te_rust(args, qrels);
            assert!(!out.status.success(), "{qrels} {args:?} was accepted");
            let stderr = String::from_utf8_lossy(&out.stderr);
            let expected = format!("Line 2: relevance level {level} is outside -127 to 127");
            assert!(stderr.contains(&expected), "{stderr}");
        }
    }
}
