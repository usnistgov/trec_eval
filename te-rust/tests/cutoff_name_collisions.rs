use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cutoff-name-collisions")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn eval(extra: &[&str]) -> Output {
    let (qrels, run) = (fixture("one.qrels"), fixture("one.run"));
    let mut args = extra.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(&args)
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

fn assert_refused(extra: &[&str], message: &str) {
    let out = eval(extra);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{extra:?} was accepted");
    assert!(out.stdout.is_empty(), "{extra:?} printed output");
    assert!(stderr.contains(message), "{extra:?}: {stderr}");
}

#[test]
fn cutoffs_that_print_the_same_row_name_are_refused() {
    // Both tools printed Rprec_mult_1.00 twice, and iprec_at_recall_0.10 twice.
    assert_refused(
        &["-m", "Rprec_mult.0.999,1.001"],
        "cutoffs 0.999 and 1.001 both print as 1.00",
    );
    assert_refused(
        &["-m", "iprec_at_recall.0.101,0.104"],
        "cutoffs 0.101 and 0.104 both print as 0.10",
    );
    assert_refused(
        &["-R", "qrels_jg", "-m", "Rprec_mult_avgjg.1.001,0.999"],
        "both print as 1.00",
    );
}

#[test]
fn cutoffs_with_distinct_names_are_unchanged() {
    // trec_eval prints the same rows and values.
    let printed = |measure: &str| -> Vec<(String, String)> {
        rows(&eval(&["-m", measure]))
            .into_iter()
            .map(|(name, _, value)| (name, value))
            .collect()
    };
    let pair = |name: &str, value: &str| (name.to_string(), value.to_string());
    assert_eq!(
        printed("Rprec_mult.0.99,1.01"),
        [
            pair("Rprec_mult_0.99", "1.0000"),
            pair("Rprec_mult_1.01", "1.0000")
        ]
    );
    assert_eq!(
        printed("iprec_at_recall.0.1,0.11"),
        [
            pair("iprec_at_recall_0.10", "1.0000"),
            pair("iprec_at_recall_0.11", "1.0000")
        ]
    );
    // Every default list prints distinct names.
    for measure in ["iprec_at_recall", "Rprec_mult"] {
        assert!(eval(&["-m", measure]).status.success(), "{measure}");
    }
}
