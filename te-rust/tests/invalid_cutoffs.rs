use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/invalid-cutoffs")
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

fn eval(measure: &str) -> Output {
    te_rust(&[
        "-m",
        measure,
        &fixture("topic.qrels"),
        &fixture("topic.run"),
    ])
}

fn assert_refused(measure: &str) {
    let out = eval(measure);
    assert!(!out.status.success(), "{measure} was accepted");
    assert!(out.stdout.is_empty(), "{measure} printed output");
}

#[test]
fn zero_rank_cutoff_is_refused_in_every_family() {
    for measure in [
        "P",
        "recall",
        "success",
        "map_cut",
        "ndcg_cut",
        "relative_P",
        "unj",
    ] {
        assert_refused(&format!("{measure}.0"));
        assert_refused(&format!("{measure}.0,5"));
    }
    assert_refused("P_0");
    assert_refused("P@0");
}

#[test]
fn duplicate_cutoffs_are_refused() {
    assert_refused("P.5,5");
    assert_refused("map_cut.2,5,2");
    assert_refused("iprec_at_recall.0.5,0.5");
    assert_refused("Rprec_mult.1.0,1.0");
}

#[test]
fn legal_lists_are_unchanged() {
    assert_eq!(summary(&eval("P.5"), "P_5"), "1.0000");
    assert_eq!(
        summary(&eval("iprec_at_recall.0.0"), "iprec_at_recall_0.00"),
        "1.0000"
    );
    // Every default list is ascending and distinct.
    for measure in [
        "P",
        "recall",
        "success",
        "map_cut",
        "ndcg_cut",
        "unj",
        "iprec_at_recall",
        "Rprec_mult",
    ] {
        assert!(eval(measure).status.success(), "{measure}");
    }
}

#[test]
fn repeated_coefficients_stay_legal_where_order_carries_meaning() {
    assert_eq!(
        summary(&eval("utility.1,-1,0,0"), "utility_1,-1,0,0"),
        "6.0000"
    );
    assert!(eval("11pt_avg.0.5,0.5").status.success());
}
