use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/relevance-level-bounds")
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

use std::time::{Duration, Instant};

/// Runs te-rust and fails instead of waiting forever on a level that does not finish.
fn te_rust_within(limit: Duration, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > limit {
            child.kill().unwrap();
            panic!("te-rust did not finish within {limit:?}: {args:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}

const LIMIT: Duration = Duration::from_secs(20);

#[test]
fn huge_level_finishes_with_nothing_relevant() {
    let (qrels, run) = (fixture("topics.qrels"), fixture("topics.run"));
    assert!(
        te_rust_within(LIMIT, &["-l", "1000000000000", &qrels, &run])
            .status
            .success()
    );
    for measure in ["bpref", "gm_bpref", "map"] {
        let out = te_rust_within(LIMIT, &["-l", "1000000000000", "-m", measure, &qrels, &run]);
        assert_eq!(summary(&out, measure), "0.0000", "{measure}");
    }
}

#[test]
fn largest_level_does_not_overflow() {
    let out = te_rust_within(
        LIMIT,
        &[
            "-l",
            "9223372036854775807",
            "-m",
            "map",
            &fixture("topics.qrels"),
            &fixture("topics.run"),
        ],
    );
    assert_eq!(summary(&out, "map"), "0.0000");
}

#[test]
fn negative_level_finishes() {
    // Whether the CLI accepts a negative level is a separate question. The library and the
    // Python Evaluator pass one to bpref and gm_bpref, which must finish.
    let (qrels, run) = (fixture("topics.qrels"), fixture("topics.run"));
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let qrels = te_rust::io::parse_trec_qrels(&qrels).unwrap();
        let run = te_rust::io::parse_trec_run(&run).unwrap();
        let requested = ["bpref".to_string(), "gm_bpref".to_string()];
        let measures = te_rust::metrics::registry::resolve_measures(&requested).unwrap();
        let config = te_rust::metrics::EvalConfig {
            relevance_level: -1,
            ..Default::default()
        };
        let out = te_rust::eval::evaluate_run(&qrels, &run, &measures, &config, None);
        done.send(out.num_queries_evaluated).unwrap();
    });
    let evaluated = finished
        .recv_timeout(LIMIT)
        .expect("bpref at level -1 did not finish");
    assert_eq!(evaluated, 2);
}

#[test]
fn bpref_is_unchanged_at_ordinary_levels() {
    // Topic 1 at -l 1: d1 and d3 relevant, one nonrelevant d2 above d3. bpref = (1 + 0) / 2.
    // Topic 2 at -l 1: d2 relevant and first. bpref = 1. Mean = 0.75.
    let out = te_rust(&[
        "-l",
        "1",
        "-m",
        "bpref",
        &fixture("topics.qrels"),
        &fixture("topics.run"),
    ]);
    assert_eq!(summary(&out, "bpref"), "0.7500");
    // At -l 2, d3 alone is relevant in topic 1 and ranks below two nonrelevant documents.
    // Topic 2 has nothing relevant. Both score 0.
    let out = te_rust(&[
        "-l",
        "2",
        "-m",
        "bpref",
        &fixture("topics.qrels"),
        &fixture("topics.run"),
    ]);
    assert_eq!(summary(&out, "bpref"), "0.0000");
}
