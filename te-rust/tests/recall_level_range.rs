use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recall-level-range")
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

/// The printed value of `measure` on the `all` row.
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

fn assert_refused(extra: &[&str], message: &str) {
    let out = eval(extra);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{extra:?} was accepted");
    assert!(out.stdout.is_empty(), "{extra:?} printed output");
    assert!(stderr.contains(message), "{extra:?}: {stderr}");
}

#[test]
fn recall_level_outside_zero_to_one_is_refused() {
    // Both tools printed iprec_at_recall_-1.00 1.0000 and iprec_at_recall_1.50 0.0000.
    for measure in ["iprec_at_recall", "11pt_avg"] {
        for levels in ["-1,1.5", "1.5", "-0.1", "0.5,1.01"] {
            assert_refused(&["-m", &format!("{measure}.{levels}")], "is outside [0, 1]");
        }
    }
}

#[test]
fn multiple_of_r_that_is_not_positive_is_refused() {
    for multiples in ["-1", "0", "-0.0", "0,1"] {
        assert_refused(
            &["-m", &format!("Rprec_mult.{multiples}")],
            "is not a positive number",
        );
        assert_refused(
            &[
                "-R",
                "qrels_jg",
                "-m",
                &format!("Rprec_mult_avgjg.{multiples}"),
            ],
            "is not a positive number",
        );
    }
}

#[test]
fn levels_and_multiples_in_range_are_unchanged() {
    // trec_eval's values. R = 1 and the relevant document is at rank 1 of 2.
    let out = eval(&["-m", "iprec_at_recall.0,1"]);
    assert_eq!(summary(&out, "iprec_at_recall_0.00"), "1.0000");
    assert_eq!(summary(&out, "iprec_at_recall_1.00"), "1.0000");
    assert_eq!(
        summary(&eval(&["-m", "11pt_avg.0,1"]), "11pt_avg_0,1"),
        "1.0000"
    );
    assert_eq!(
        summary(&eval(&["-m", "Rprec_mult.2"]), "Rprec_mult_2.00"),
        "0.5000"
    );
    assert_eq!(
        summary(
            &eval(&["-R", "qrels_jg", "-m", "Rprec_mult_avgjg.2"]),
            "Rprec_mult_avgjg_2.00"
        ),
        "0.5000"
    );
    for measure in ["iprec_at_recall", "11pt_avg", "Rprec_mult"] {
        assert!(eval(&["-m", measure]).status.success(), "{measure}");
    }
}
