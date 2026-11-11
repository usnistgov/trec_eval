use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/empty-param-lists")
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

#[test]
fn empty_parameter_list_is_refused() {
    for measure in [
        "P.",
        "P@",
        "P_",
        "recall_",
        "map_cut_",
        "ndcg_cut_",
        "success_",
        "unj_",
        "P_avgjg_",
    ] {
        let out = eval(measure);
        assert!(!out.status.success(), "{measure} was accepted");
        assert!(out.stdout.is_empty(), "{measure} printed output");
    }
}

#[test]
fn trailing_dot_in_a_number_is_not_an_empty_list() {
    // 1. is a legal float.
    assert_eq!(
        summary(&eval("iprec_at_recall.1."), "iprec_at_recall_1.00"),
        "1.0000"
    );
    assert_eq!(rows(&eval("Rprec_mult.1.,2.")).len(), 2);
    assert!(eval("rbp.p=1.").status.success());
    assert!(eval("ndcg.1=2.").status.success());
}

#[test]
fn measures_without_parameters_are_unchanged() {
    assert!(eval("P").status.success());
    assert!(eval("map").status.success());
    assert_eq!(summary(&eval("P.5"), "P_5"), "1.0000");
}
