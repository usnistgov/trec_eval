//! The differences from trec_eval that docs/cli.md lists. A failure here means
//! that list needs updating.

use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

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
        .stdin(File::open(test_data("results.test")).unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run te-rust")
}

fn eval(options: &[&str], run: &str) -> Output {
    let qrels = test_data("qrels.test");
    let mut args = options.to_vec();
    args.push(&qrels);
    args.push(run);
    te_rust(&args)
}

#[test]
fn listed_trec_eval_features_are_refused() {
    let run = test_data("results.test");
    for options in [
        &["-T", "trec_results"][..],
        &["-D", "1"],
        &["-Z", "zscores"],
        &["-o"],
        &["--Max_retrieved_per_topic=5"],
        &["--complete_rel_info_wanted"],
        &["-m", "prefs_simp"],
        &["-R", "prefs"],
        &["-R", "qrels_prefs"],
    ] {
        assert!(!eval(options, &run).status.success(), "{options:?}");
    }
    // trec_eval reads the run from standard input when its name is "-".
    assert!(!eval(&["-m", "map"], "-").status.success());
    assert!(!te_rust(&["-v"]).status.success());
    assert!(te_rust(&["-V"]).status.success());
}

#[test]
fn listed_extensions_are_accepted() {
    let run = test_data("results.test");
    for (measure, row) in [
        ("P_5", "P_5"),
        ("P_avgjg_5", "P_avgjg_5"),
        ("recall_10", "recall_10"),
        ("ndcg_cut_10", "ndcg_cut_10"),
        ("map_cut_100", "map_cut_100"),
        ("success_5", "success_5"),
        ("unj_10", "unj_10"),
        ("P@5", "P_5"),
        ("recall@10", "recall_10"),
        ("success@5", "success_5"),
        ("unj@10", "unj_10"),
        ("relative_P@5", "relative_P_5"),
        ("ndcg@10", "ndcg_cut_10"),
        ("map@100", "map_cut_100"),
        ("mrr", "recip_rank"),
        ("rr", "recip_rank"),
        ("precision.5", "P_5"),
        ("MAP", "map"),
        ("ndcg_p", "ndcg_p"),
    ] {
        let output = eval(&["-m", measure], &run);
        assert!(output.status.success(), "{measure}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout
                .lines()
                .any(|line| line.split_whitespace().next() == Some(row)),
            "{measure} did not print {row}: {stdout}"
        );
    }
}

#[test]
fn rel_format_help_lists_only_supported_formats() {
    let output = te_rust(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|line| line.contains("--rel-format"))
        .expect("no -R line in --help");
    assert!(line.contains("qrels_jg"), "{line}");
    assert!(!line.contains("prefs"), "{line}");
}

/// docs/cli.md lists `-l=2` as a te-rust spelling of `-l 2`.
#[test]
fn short_option_value_after_equals_is_read_like_a_separate_value() {
    let run = test_data("results.test");
    for (joined, separate) in [
        (&["-l=2", "-m", "map"][..], &["-l", "2", "-m", "map"][..]),
        (&["-M=1", "-m", "P.1,2"], &["-M", "1", "-m", "P.1,2"]),
        (
            &["-N=10", "-m", "utility.0,0,0,1"],
            &["-N", "10", "-m", "utility.0,0,0,1"],
        ),
        (&["-m=recip_rank"], &["-m", "recip_rank"]),
        (&["-R=qrels", "-m", "map"], &["-R", "qrels", "-m", "map"]),
    ] {
        let (joined_out, separate_out) = (eval(joined, &run), eval(separate, &run));
        assert!(separate_out.status.success(), "{separate:?}");
        assert!(!separate_out.stdout.is_empty(), "{separate:?}");
        assert_eq!(
            joined_out.status.code(),
            separate_out.status.code(),
            "{joined:?}"
        );
        assert_eq!(joined_out.stdout, separate_out.stdout, "{joined:?}");
    }
}
