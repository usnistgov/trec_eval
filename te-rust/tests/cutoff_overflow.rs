use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cutoff-overflow")
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

#[test]
fn cutoff_above_i64_max_is_refused() {
    // trec_eval: "Error: value `9223372036854775808' out of range." and exit 2.
    // te-rust printed P_9223372036854775808 0.0000.
    for measure in [
        "P",
        "recall",
        "map_cut",
        "ndcg_cut",
        "relative_P",
        "success",
        "unj",
    ] {
        for cutoff in ["9223372036854775808", "18446744073709551615"] {
            let out = eval(&["-m", &format!("{measure}.{cutoff}")]);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(!out.status.success(), "{measure}.{cutoff} was accepted");
            assert!(out.stdout.is_empty(), "{measure}.{cutoff} printed output");
            assert!(
                stderr.contains("out of range"),
                "{measure}.{cutoff}: {stderr}"
            );
        }
    }
    let out = eval(&["-R", "qrels_jg", "-m", "P_avgjg.9223372036854775808"]);
    assert!(!out.status.success());
}

#[test]
fn cutoff_at_i64_max_is_accepted() {
    // trec_eval prints the same values: one relevant document in a ranking of two.
    let max = "9223372036854775807";
    assert_eq!(
        summary(&eval(&["-m", &format!("P.{max}")]), &format!("P_{max}")),
        "0.0000"
    );
    assert_eq!(
        summary(
            &eval(&["-m", &format!("map_cut.{max}")]),
            &format!("map_cut_{max}")
        ),
        "1.0000"
    );
}
