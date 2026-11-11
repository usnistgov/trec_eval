use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rbp-gain-map")
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

#[test]
fn gain_map_in_the_measure_parameters_is_applied() {
    // Gains 1 -> 3, 2 -> 9, 3 -> 3 normalise by 9: d = 1/3, a = 1, c = 1/3, at ranks 1, 3, 5.
    // 0.1 * (1/3 + 1 * 0.81 + 1/3 * 0.6561) = 0.1362
    let out = te_rust(&[
        "-m",
        "rbp.1=3,2=9",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp_1=3,2=9"), "0.1362");
}

#[test]
fn p_and_gain_map_combine() {
    // 0.5 * (1/3 + 1 * 0.25 + 1/3 * 0.0625) = 0.3021
    let out = te_rust(&[
        "-m",
        "rbp.p=0.5,1=3,2=9",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp_p=0.5,1=3,2=9"), "0.3021");
}

#[test]
fn p_alone_keeps_default_gains() {
    // 0.5 * (1 + 2/3 * 0.25 + 1/3 * 0.0625) = 0.5938
    let out = te_rust(&[
        "-m",
        "rbp.p=0.5",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp_p=0.5"), "0.5938");
}

#[test]
fn global_gains_still_apply() {
    let out = te_rust(&[
        "--global-gains",
        "1=3,2=9",
        "-m",
        "rbp",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp"), "0.1362");
}

#[test]
fn parameter_gains_override_global_gains() {
    // The same precedence as ndcg.
    let out = te_rust(&[
        "--global-gains",
        "2=1,1=9",
        "-m",
        "rbp.1=3,2=9",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp_1=3,2=9"), "0.1362");
}

#[test]
fn default_measure_is_unchanged() {
    // 0.1 * (1 + 2/3 * 0.81 + 1/3 * 0.6561) = 0.1759
    let out = te_rust(&[
        "-m",
        "rbp",
        &fixture("graded.qrels"),
        &fixture("graded.run"),
    ]);
    assert_eq!(summary(&out, "rbp"), "0.1759");
}
