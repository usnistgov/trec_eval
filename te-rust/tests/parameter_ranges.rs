use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/parameter-ranges")
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
fn negative_set_f_weight_is_refused() {
    // Both tools printed set_F_-0.5 with a value, from (x + 1) * P * R / (x * P + R).
    assert_refused(&["-m", "set_F.-0.5"], "set_F weight -0.5 is negative");
    assert_refused(&["-m", "set_F.-1e-9"], "is negative");
}

#[test]
fn persistence_outside_zero_to_one_is_refused() {
    // rbp multiplies its sum by 1 - p, and rbp.p=1 printed 0.0000 for every run.
    for measure in ["rbp", "rbp_resid"] {
        for p in ["1", "1.5", "-0.5", "-0.1"] {
            assert_refused(
                &["-m", &format!("{measure}.p={p}")],
                &format!("persistence p {p} is outside [0, 1)"),
            );
        }
    }
    assert_refused(&["-m", "rbp.p=0.5,1=2,p=2"], "is outside [0, 1)");
}

#[test]
fn values_at_the_bounds_are_unchanged() {
    // One relevant document, a, at rank 1 of 2, so P = 1/2 and R = 1. set_F.x gives
    // (x + 1) * P * R / (x * P + R): x = 0 gives P = 0.5000, the default x = 1 gives
    // 1 / 1.5 = 0.6667, and x = 0.5 gives 0.75 / 1.25 = 0.6000.
    assert_eq!(summary(&eval(&["-m", "set_F.0"]), "set_F_0"), "0.5000");
    assert_eq!(summary(&eval(&["-m", "set_F"]), "set_F"), "0.6667");
    assert_eq!(summary(&eval(&["-m", "set_F.0.5"]), "set_F_0.5"), "0.6000");
    // rbp is (1 - p) * 1 for a relevant document at rank 1: 1.0000 at p = 0 and 0.5000 at
    // p = 0.5. rbp_resid counts unjudged documents, and this run has none.
    assert_eq!(summary(&eval(&["-m", "rbp.p=0"]), "rbp_p=0"), "1.0000");
    assert_eq!(summary(&eval(&["-m", "rbp.p=0.5"]), "rbp_p=0.5"), "0.5000");
    assert_eq!(
        summary(&eval(&["-m", "rbp_resid.p=0"]), "rbp_resid_p=0"),
        "0.0000"
    );
}
