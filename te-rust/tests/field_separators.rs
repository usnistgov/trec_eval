use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/field-separators")
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

fn relstring(output: &Output) -> String {
    rows(output)
        .into_iter()
        .find(|(name, topic, _)| name == "relstring" && topic == "1")
        .unwrap()
        .2
}

#[test]
fn nbsp_stays_inside_the_docno() {
    // The first line is docno "d<NBSP>1", score 0.5, which is unjudged. x has score 0.9,
    // so x ranks first. Splitting at NBSP gives docno "d" and reads 9 as its score.
    let out = te_rust(&[
        "-q",
        "-m",
        "relstring",
        &fixture("judged.qrels"),
        &fixture("nbsp-docno.run"),
    ]);
    assert_eq!(relstring(&out), "'1-'");
}

#[test]
fn ideographic_space_stays_inside_the_docno() {
    let out = te_rust(&[
        "-q",
        "-m",
        "relstring",
        &fixture("judged.qrels"),
        &fixture("ideographic-docno.run"),
    ]);
    assert_eq!(relstring(&out), "'-'");
}

#[test]
fn nbsp_docno_matches_between_qrels_and_run() {
    let out = te_rust(&[
        "-q",
        "-m",
        "relstring",
        &fixture("nbsp-docno.qrels"),
        &fixture("nbsp-docno.run"),
    ]);
    assert_eq!(relstring(&out), "'01'");
}

#[test]
fn next_line_is_not_a_separator() {
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("nel-separated.qrels"),
        &fixture("d1.run"),
    ]);
    assert!(!out.status.success());
}

#[test]
fn qrels_jg_parser_keeps_nbsp_in_the_docno() {
    // x has score 0.9 and is nonrelevant. "d<NBSP>1" at rank 2 is relevant: AP = 1/2.
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-m",
        "map_avgjg",
        &fixture("nbsp-docno.qrels_jg"),
        &fixture("nbsp-docno.run"),
    ]);
    assert_eq!(summary(&out, "map_avgjg"), "0.5000");
}

#[test]
fn leading_nbsp_belongs_to_the_run_qid() {
    // Trimmed, the first line would repeat document x in topic 1, which is refused.
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("x.qrels"),
        &fixture("leading-nbsp.run"),
    ]);
    assert_eq!(summary(&out, "map"), "1.0000");
}

#[test]
fn trailing_nbsp_belongs_to_the_run_id() {
    let out = te_rust(&[
        "-m",
        "runid",
        &fixture("x.qrels"),
        &fixture("trailing-nbsp.run"),
    ]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("r\u{a0}"));
}

#[test]
fn leading_nbsp_belongs_to_the_qrels_qid() {
    // Topic "<NBSP>1" has no results, so topic 1 keeps its one relevant document.
    // Trimmed, z would join topic 1 and num_rel would be 2.
    let out = te_rust(&[
        "-m",
        "num_rel",
        &fixture("leading-nbsp.qrels"),
        &fixture("x.run"),
    ]);
    assert_eq!(summary(&out, "num_rel"), "1");
    let out = te_rust(&[
        "-R",
        "qrels_jg",
        "-m",
        "num_rel",
        &fixture("leading-nbsp.qrels_jg"),
        &fixture("x.run"),
    ]);
    assert_eq!(summary(&out, "num_rel"), "1");
}

#[test]
fn trailing_nbsp_is_part_of_the_grade() {
    // The grade field is "1<NBSP>". As in trec_eval, its leading integer is read with a warning.
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("trailing-nbsp.qrels"),
        &fixture("x.run"),
    ]);
    assert_eq!(summary(&out, "map"), "1.0000");
    assert!(String::from_utf8_lossy(&out.stderr).contains("partially read as '1'"));
}

#[test]
fn every_ascii_separator_still_splits() {
    // Vertical tab, form feed, tab and a CRLF ending. A CR without LF is refused, because it
    // ends the lines of a file with CR line endings (line_bytes.rs).
    let out = te_rust(&[
        "-m",
        "map",
        &fixture("ascii-separators.qrels"),
        &fixture("d1.run"),
    ]);
    assert_eq!(summary(&out, "map"), "1.0000");
}
