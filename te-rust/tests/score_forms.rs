//! Scores in the forms that trec_eval's `strtod` reads (#125). In abc.qrels the grades of a, b
//! and c are 1, 2 and 3, so `relstring` spells the ranking: '213' puts b first. In each run, a
//! scores 5 and c scores 3, and b's score is the case under test.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/score-forms")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(run: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args([
            "-q",
            "-m",
            "relstring",
            &fixture("abc.qrels"),
            &fixture(run),
        ])
        .output()
        .expect("failed to run te-rust")
}

/// The relstring of topic 1.
fn relstring(output: &Output) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 3 && fields[0] == "relstring" && fields[1] == "1")
        .expect("no relstring row for topic 1")[2]
        .to_string()
}

#[test]
fn infinite_and_out_of_range_scores_rank_first_or_last() {
    // trec_eval prints the same relstrings for these runs.
    for (run, expected) in [
        ("inf.run", "'213'"),
        ("infinity.run", "'213'"),
        ("1e400.run", "'213'"),
        ("minus-inf.run", "'132'"),
    ] {
        assert_eq!(relstring(&te_rust(run)), expected, "{run}");
    }
}

#[test]
fn tied_infinite_scores_fall_back_to_docno_order() {
    // a and b both score inf. As for any tie, the docno that sorts last ranks first: b, a, c.
    assert_eq!(relstring(&te_rust("tied-inf.run")), "'213'");
}

#[test]
fn hexadecimal_scores_are_read() {
    // 0x10 is 16 and 0x1p3 is 8, above a's 5; 0x1p-2 is 0.25, below c's 3.
    for (run, expected) in [
        ("hex-16.run", "'213'"),
        ("hex-8.run", "'213'"),
        ("hex-quarter.run", "'132'"),
    ] {
        assert_eq!(relstring(&te_rust(run)), expected, "{run}");
    }
}

#[test]
fn a_score_with_trailing_characters_is_read_in_part_with_a_warning() {
    // 1.5x reads as 1.5, last of the three. 5e reads as 5 and ties with a, so b ranks first.
    // The warnings carry trec_eval's wording with #125.
    let out = te_rust("trailing-x.run");
    assert_eq!(relstring(&out), "'132'");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Line 2: score '1.5x' partially read as '1.5'"),
        "{stderr}"
    );
    let out = te_rust("trailing-e.run");
    assert_eq!(relstring(&out), "'213'");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Line 2: score '5e' partially read as '5'"),
        "{stderr}"
    );
}

#[test]
fn a_score_outside_the_f64_range_is_kept_with_a_warning() {
    // trec_eval's reader keeps strtod's value, warns when strtod sets ERANGE and gives +-inf or
    // +-0, and breaks the ties that follow by docno: b, a, c for 1e401, 1e400 and -1e400, and c,
    // b, a for 2e-400, 1e-400 and -1e-400.
    for (run, expected, warnings) in [
        (
            "over.run",
            "'213'",
            [
                "Line 1: score '1e401' out of range, read as inf",
                "Line 2: score '1e400' out of range, read as inf",
                "Line 3: score '-1e400' out of range, read as -inf",
            ],
        ),
        (
            "under.run",
            "'321'",
            [
                "Line 1: score '2e-400' out of range, read as 0",
                "Line 2: score '1e-400' out of range, read as 0",
                "Line 3: score '-1e-400' out of range, read as -0",
            ],
        ),
    ] {
        let out = te_rust(run);
        assert_eq!(relstring(&out), expected, "{run}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        for warning in warnings {
            assert!(stderr.contains(warning), "{run}: {stderr}");
        }
    }
}

#[test]
fn a_subnormal_score_is_kept_without_a_warning() {
    // 2e-310 > 1e-310 > 0, all representable: a, b, c, as in trec_eval.
    let out = te_rust("subnormal.run");
    assert_eq!(relstring(&out), "'123'");
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn nan_and_scores_without_a_number_are_refused() {
    for (run, value) in [("nan.run", "'nan'"), ("abc.run", "'abc'")] {
        let out = te_rust(run);
        assert!(!out.status.success(), "{run} was accepted");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(&format!("Line 2: invalid float for field 'sim': {value}")),
            "{stderr}"
        );
    }
}
