use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/argument-prefix")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// A copy of a file from the C tree's `test/` directory.
fn test_data(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/test-data")
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

fn eval(options: &[&str]) -> Output {
    let (qrels, run) = (fixture("graded.qrels"), fixture("graded.run"));
    let mut args = options.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    te_rust(&args)
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// `options` scores like `integer`, and warns that `value` was read as `read`.
fn assert_reads_as(options: &[&str], integer: &[&str], value: &str, read: &str) {
    let (partial, whole) = (eval(options), eval(integer));
    assert_eq!(stdout(&partial), stdout(&whole), "{options:?}");
    let warning = format!("value '{value}' partially read as '{read}'");
    let stderr = String::from_utf8_lossy(&partial.stderr);
    assert!(stderr.contains(&warning), "{options:?}: {stderr}");
    assert!(whole.stderr.is_empty(), "{integer:?}");
}

fn assert_refused(options: &[&str], message: &str) {
    let out = eval(options);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{options:?} was accepted");
    assert!(out.stdout.is_empty(), "{options:?} printed output");
    assert!(stderr.contains(message), "{options:?}: {stderr}");
}

#[test]
fn cutoffs_read_their_leading_integer_with_a_warning() {
    // trec_eval: "Warning, value `5x' partially read as `5'." and P_5. te-rust refused both.
    assert_reads_as(&["-m", "P.5x"], &["-m", "P.5"], "5x", "5");
    assert_reads_as(&["-m", "P.1.5"], &["-m", "P.1"], "1.5", "1");
    assert_reads_as(&["-m", "map_cut.2,3x"], &["-m", "map_cut.2,3"], "3x", "3");
}

#[test]
fn relevance_level_and_document_counts_read_their_leading_integer_with_a_warning() {
    // b (grade 2) is at rank 2: at level 2, map = 1/2; at level 1, map = (1/2 + 2/3) / 2.
    assert_reads_as(
        &["-l", "2.5", "-m", "map"],
        &["-l", "2", "-m", "map"],
        "2.5",
        "2",
    );
    assert_reads_as(
        &["-M", "2x", "-m", "map"],
        &["-M", "2", "-m", "map"],
        "2x",
        "2",
    );
    assert_reads_as(
        &["-N", "10x", "-m", "utility.0,0,0,1"],
        &["-N", "10", "-m", "utility.0,0,0,1"],
        "10x",
        "10",
    );
}

#[test]
fn values_without_a_leading_integer_are_refused() {
    // trec_eval: "could not be parsed", and "out of range" for a value beyond a long.
    assert_refused(&["-m", "P.x"], "invalid integer cutoff 'x'");
    assert_refused(&["-m", "P..5"], "invalid integer cutoff '.5'");
    assert_refused(&["-l", ".5", "-m", "map"], "value '.5' could not be parsed");
    assert_refused(&["-l", "x", "-m", "map"], "value 'x' could not be parsed");
    assert_refused(
        &["-l", "9223372036854775808", "-m", "map"],
        "value '9223372036854775808' out of range",
    );
    assert_refused(&["-M", "x", "-m", "map"], "value 'x' could not be parsed");
    assert_refused(&["-M-1", "-m", "map"], "value '-1' is negative");
    assert_refused(&["-N", "x", "-m", "map"], "value 'x' could not be parsed");
}

#[test]
fn makefile_meas_params_check_reads_23_4_as_23() {
    // The Makefile's quicktest runs `-mndcg_cut.10,20,23.4 -msuccess.2,5,20` and expects
    // test/out.test.meas_params, which has ndcg_cut_23.
    let out = te_rust(&[
        "-q",
        "-mndcg_cut.10,20,23.4",
        "-msuccess.2,5,20",
        &test_data("qrels.test"),
        &test_data("results.test"),
    ]);
    let expected = std::fs::read_to_string(test_data("out.test.meas_params")).unwrap();
    let rows = |text: &str| -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = text
            .lines()
            .map(|line| line.split_whitespace().map(str::to_string).collect())
            .filter(|fields: &Vec<String>| {
                fields[0].starts_with("ndcg_cut_") || fields[0].starts_with("success_")
            })
            .collect();
        rows.sort();
        rows
    };
    let actual = rows(&stdout(&out));
    assert_eq!(actual.len(), 24);
    assert_eq!(actual, rows(&expected));
}
