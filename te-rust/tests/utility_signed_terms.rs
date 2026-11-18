use std::path::PathBuf;

use te_rust::eval::evaluate_run;
use te_rust::io::{parse_trec_qrels, parse_trec_run};
use te_rust::metrics::utility::UtilityMeasure;
use te_rust::metrics::{EvalConfig, Measure, MetricValue};

fn path(dir: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(dir)
        .join(name)
}

/// The summary value of utility with `coeffs`, at relevance level `level`.
fn utility(qrels: PathBuf, run: PathBuf, coeffs: [f64; 4], level: i64, num_docs: usize) -> String {
    let qrels = parse_trec_qrels(qrels).unwrap();
    let run = parse_trec_run(run).unwrap();
    let measures: Vec<Box<dyn Measure>> = vec![Box::new(UtilityMeasure::new(coeffs.to_vec(), ""))];
    let config = EvalConfig {
        relevance_level: level,
        num_docs_in_coll: num_docs,
        ..EvalConfig::default()
    };
    let out = evaluate_run(&qrels, &run, &measures, &config, None);
    match out.summary_results[0].value {
        MetricValue::Float(v) => format!("{v:.4}"),
        ref other => panic!("{other:?}"),
    }
}

#[test]
fn unjudged_documents_do_not_count_as_relevant_retrieved() {
    // qrels a 1, b 0; run a, x, y. At level -1, a and b are relevant, and x and y are not in
    // the qrels. Retrieved relevant 1 (a), retrieved nonrelevant 2, missed relevant 1 (b), so
    // 1 - 2 + 1 = 0, and with 10 documents d = 10 + 1 - 3 - 2 = 6, as trec_eval -l-1 prints.
    // te-rust counted x and y as relevant retrieved, so 2 - 3 wrapped in usize: a debug build
    // panicked and a release build printed 18446744073709551616.
    let (qrels, run) = (
        || path("fixtures/utility-signed-terms", "one.qrels"),
        || path("fixtures/utility-signed-terms", "axy.run"),
    );
    assert_eq!(
        utility(qrels(), run(), [1.0, -1.0, 1.0, 0.0], -1, 0),
        "0.0000"
    );
    assert_eq!(
        utility(qrels(), run(), [0.0, 0.0, 0.0, 1.0], -1, 10),
        "6.0000"
    );
    // Levels 0 and 1 are unchanged. At 0, a and b are relevant: 1 - 2 + 1 = 0. At 1, only a
    // is: 1 - 2 + 0 = -1. trec_eval prints the same.
    assert_eq!(
        utility(qrels(), run(), [1.0, -1.0, 1.0, 0.0], 0, 0),
        "0.0000"
    );
    assert_eq!(
        utility(qrels(), run(), [1.0, -1.0, 1.0, 0.0], 1, 0),
        "-1.0000"
    );
}

#[test]
fn negative_level_matches_trec_eval_on_its_test_files() {
    // trec_eval -l-1 -m utility test/qrels.test test/results.test prints -8.0000, and with
    // -m utility.2,-1,1,0.5 -N100000 prints 50478.5000. te-rust printed 500.0000 and
    // 51113.5000.
    let (qrels, run) = (
        || path("test-data", "qrels.test"),
        || path("test-data", "results.test"),
    );
    assert_eq!(
        utility(qrels(), run(), [1.0, -1.0, 0.0, 0.0], -1, 0),
        "-8.0000"
    );
    assert_eq!(
        utility(qrels(), run(), [2.0, -1.0, 1.0, 0.5], -1, 100_000),
        "50478.5000"
    );
}
