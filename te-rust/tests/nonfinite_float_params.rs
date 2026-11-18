use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nonfinite-float-params")
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

fn assert_refused(extra: &[&str]) {
    let out = eval(extra);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{extra:?} was accepted");
    assert!(out.stdout.is_empty(), "{extra:?} printed output");
    assert!(
        stderr.contains("is not a finite number"),
        "{extra:?}: {stderr}"
    );
}

#[test]
fn nan_and_infinite_parameters_are_refused() {
    // te-rust printed set_F_nan 0.0000 and set_F_inf NaN, where trec_eval prints nan for both.
    for value in ["nan", "NaN", "inf", "-inf", "infinity", "1e400"] {
        assert_refused(&["-m", &format!("set_F.{value}")]);
        assert_refused(&["-m", &format!("rbp.p={value}")]);
        assert_refused(&["-m", &format!("rbp_resid.p={value}")]);
        assert_refused(&["-m", &format!("iprec_at_recall.{value}")]);
        assert_refused(&["-m", &format!("11pt_avg.{value}")]);
        assert_refused(&["-m", &format!("utility.{value},-1,0,0")]);
    }
    assert_refused(&["-R", "qrels_jg", "-m", "Rprec_mult_avgjg.nan"]);
}

#[test]
fn nan_cannot_change_the_value_at_another_cutoff() {
    // R = 1 and the relevant document is at rank 1, so Rprec_mult_0.50 is 1. With the sorted
    // cutoff lists, Rprec_mult.nan,0.5 printed 0.0000 for it.
    assert_refused(&["-m", "Rprec_mult.nan,0.5"]);
    assert_eq!(
        summary(&eval(&["-m", "Rprec_mult.0.5"]), "Rprec_mult_0.50"),
        "1.0000"
    );
}

#[test]
fn finite_parameters_are_unchanged() {
    // The values are trec_eval's on the same files.
    assert_eq!(summary(&eval(&["-m", "set_F.0.5"]), "set_F_0.5"), "0.6000");
    assert_eq!(summary(&eval(&["-m", "rbp.p=0.5"]), "rbp_p=0.5"), "0.5000");
    assert_eq!(
        summary(
            &eval(&["-m", "iprec_at_recall.0.5"]),
            "iprec_at_recall_0.50"
        ),
        "1.0000"
    );
    assert_eq!(
        summary(&eval(&["-m", "11pt_avg.0.5"]), "11pt_avg_0.5"),
        "1.0000"
    );
    assert_eq!(
        summary(&eval(&["-m", "utility.1,-1,0,0"]), "utility_1,-1,0,0"),
        "0.0000"
    );
}
