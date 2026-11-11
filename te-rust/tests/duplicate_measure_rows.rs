use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/duplicate-measure-rows")
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

fn names(extra: &[&str]) -> Vec<String> {
    let (qrels, run) = (fixture("topic.qrels"), fixture("topic.run"));
    let mut args = extra.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    rows(&te_rust(&args))
        .into_iter()
        .map(|row| format!("{} {}", row.0, row.1))
        .collect()
}

fn assert_each_once(extra: &[&str]) {
    let all = names(extra);
    let distinct: std::collections::HashSet<_> = all.iter().collect();
    assert_eq!(all.len(), distinct.len(), "{extra:?}: {all:?}");
}

#[test]
fn repeated_measure_prints_once() {
    assert_eq!(names(&["-m", "map", "-m", "map"]), ["map all"]);
    // All six documents are relevant and retrieved, so the single row is 1.
    let (qrels, run) = (fixture("topic.qrels"), fixture("topic.run"));
    let out = te_rust(&["-m", "map", "-m", "map", &qrels, &run]);
    assert_eq!(summary(&out, "map"), "1.0000");
}

#[test]
fn group_and_member_print_once() {
    assert_each_once(&["-m", "official", "-m", "map"]);
    assert_each_once(&["-m", "map", "-m", "official"]);
    assert_each_once(&["-m", "official", "-m", "P.5"]);
}

#[test]
fn aliases_and_reordered_lists_print_once() {
    assert_each_once(&["-m", "P.5", "-m", "P_5", "-m", "P@5"]);
    assert_each_once(&["-m", "P.5,10", "-m", "P.10,5"]);
}

#[test]
fn a_request_covered_by_an_earlier_one_adds_nothing() {
    // trec_eval reads only the first -m P here, which prints P_2 and P_5.
    assert_eq!(names(&["-m", "P.2,5", "-m", "P.5"]), ["P_2 all", "P_5 all"]);
}

#[test]
fn per_topic_rows_print_once() {
    assert_each_once(&["-q", "-m", "map", "-m", "map"]);
}

#[test]
fn one_list_can_still_print_a_name_twice() {
    // 0.999 and 1.001 are distinct multiples that both print as 1.00, as in trec_eval.
    let printed = names(&["-m", "Rprec_mult.0.999,1.001"]);
    assert_eq!(printed, ["Rprec_mult_1.00 all", "Rprec_mult_1.00 all"]);
}

#[test]
fn a_request_with_new_rows_still_prints() {
    assert_eq!(names(&["-m", "P.2", "-m", "P.3"]), ["P_2 all", "P_3 all"]);
}
