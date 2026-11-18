use std::path::PathBuf;

use te_rust::eval::evaluate_run;
use te_rust::io::{parse_trec_qrels, parse_trec_run};
use te_rust::metrics::iprec_at_recall::IprecAtRecallMeasure;
use te_rust::metrics::map_cut::MapCutMeasure;
use te_rust::metrics::ndcg_cut::NdcgCutMeasure;
use te_rust::metrics::precision::PrecisionCutMeasure;
use te_rust::metrics::precision_avgjg::PrecisionAvgjgMeasure;
use te_rust::metrics::recall::RecallCutMeasure;
use te_rust::metrics::relative_p::RelativePMeasure;
use te_rust::metrics::rprec_mult::RprecMultMeasure;
use te_rust::metrics::rprec_mult_avgjg::RprecMultAvgjgMeasure;
use te_rust::metrics::success::SuccessCutMeasure;
use te_rust::metrics::unj::UnjMeasure;
use te_rust::metrics::{EvalConfig, Measure, MetricValue};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/constructor-cutoffs")
        .join(name)
}

/// Summary rows as (name, value to four decimals), in output order.
fn summary(name: &str, measure: Box<dyn Measure>) -> Vec<(String, String)> {
    let qrels = parse_trec_qrels(fixture(&format!("{name}.qrels"))).unwrap();
    let run = parse_trec_run(fixture(&format!("{name}.run"))).unwrap();
    let out = evaluate_run(&qrels, &run, &[measure], &EvalConfig::default(), None);
    out.summary_results
        .into_iter()
        .map(|row| match row.value {
            MetricValue::Float(v) => (row.name, format!("{v:.4}")),
            other => panic!("{}: {other:?}", row.name),
        })
        .collect()
}

#[test]
fn map_cut_from_an_unsorted_list_matches_hand_calculation() {
    // Six relevant documents, all retrieved in the top six. map_cut_2 = (1 + 1) / 6 and
    // map_cut_5 = 5 / 6. With the list kept as given, map_cut_2 took the value at the end of
    // the ranking.
    let rows = summary("six-relevant", Box::new(MapCutMeasure::new(vec![5, 2])));
    let expected = [("map_cut_2", "0.3333"), ("map_cut_5", "0.8333")];
    assert_eq!(rows, expected.map(|(n, v)| (n.to_string(), v.to_string())));
}

#[test]
fn every_constructor_sorts_its_cutoffs() {
    let pairs: Vec<(Box<dyn Measure>, Box<dyn Measure>)> = vec![
        (
            Box::new(MapCutMeasure::new(vec![5, 2])),
            Box::new(MapCutMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(NdcgCutMeasure::new(vec![5, 2])),
            Box::new(NdcgCutMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(RelativePMeasure::new(vec![5, 2])),
            Box::new(RelativePMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(UnjMeasure::new(vec![5, 2])),
            Box::new(UnjMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(PrecisionCutMeasure::new(vec![5, 2])),
            Box::new(PrecisionCutMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(RecallCutMeasure::new(vec![5, 2])),
            Box::new(RecallCutMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(SuccessCutMeasure::new(vec![5, 2])),
            Box::new(SuccessCutMeasure::new(vec![2, 5])),
        ),
        (
            Box::new(IprecAtRecallMeasure::new(vec![0.8, 0.2])),
            Box::new(IprecAtRecallMeasure::new(vec![0.2, 0.8])),
        ),
        (
            Box::new(RprecMultMeasure::new(vec![2.0, 0.5])),
            Box::new(RprecMultMeasure::new(vec![0.5, 2.0])),
        ),
    ];
    for (given, sorted) in pairs {
        let name = given.name();
        assert_eq!(summary("mixed", given), summary("mixed", sorted), "{name}");
    }
    // The qrels_jg measures print their rows in the same ascending order.
    assert_eq!(
        PrecisionAvgjgMeasure::new(vec![5, 2]).sub_metrics(),
        ["P_avgjg_2", "P_avgjg_5"]
    );
    assert_eq!(
        RprecMultAvgjgMeasure::new(vec![2.0, 0.5]).sub_metrics(),
        ["Rprec_mult_avgjg_0.50", "Rprec_mult_avgjg_2.00"]
    );
}

#[test]
#[should_panic(expected = "rank cutoff must be at least 1")]
fn zero_rank_cutoff_panics() {
    MapCutMeasure::new(vec![5, 0]);
}

#[test]
#[should_panic(expected = "duplicate cutoffs")]
fn repeated_rank_cutoff_panics() {
    PrecisionCutMeasure::new(vec![5, 2, 5]);
}

#[test]
#[should_panic(expected = "duplicate cutoffs")]
fn repeated_float_cutoff_panics() {
    RprecMultMeasure::new(vec![1.0, 0.5, 1.0]);
}
