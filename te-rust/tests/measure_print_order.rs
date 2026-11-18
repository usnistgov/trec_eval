use std::path::PathBuf;
use std::process::Command;

// topics.qrels judges topic 1 (d1 1, d2 0, d3 2) and topic 2 (d1 0, d2 1), and
// topics.run ranks d1, d2, d3 for topic 1 and d2, d1 for topic 2.
fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/measure-print-order")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// The (measure, topic) of each printed row, in output order.
fn printed_rows(args: &[&str]) -> Vec<(String, String)> {
    let output = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .arg(fixture("topics.qrels"))
        .arg(fixture("topics.run"))
        .output()
        .expect("failed to run te-rust");
    assert!(
        output.status.success(),
        "te-rust {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let mut fields = line.split_whitespace();
            let measure = fields.next().unwrap_or_default().to_string();
            let topic = fields.next().unwrap_or_default().to_string();
            (measure, topic)
        })
        .collect()
}

fn rows(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(measure, topic)| (measure.to_string(), topic.to_string()))
        .collect()
}

#[test]
fn measures_print_in_trec_eval_table_order() {
    // trec_eval prints num_q before map, whatever the order of the -m options.
    assert_eq!(
        printed_rows(&["-m", "map", "-m", "num_q"]),
        rows(&[("num_q", "all"), ("map", "all")])
    );
    // trec_eval's output for the reversed list.
    assert_eq!(
        printed_rows(&["-m", "unj.5", "-m", "rbp", "-m", "P.5", "-m", "map", "-m", "runid"]),
        rows(&[
            ("runid", "all"),
            ("map", "all"),
            ("P_5", "all"),
            ("rbp", "all"),
            ("unj_5", "all"),
        ])
    );
}

#[test]
fn topic_rows_follow_the_same_order() {
    // trec_eval -q prints map before recip_rank in each topic and in the summary.
    assert_eq!(
        printed_rows(&["-q", "-m", "recip_rank", "-m", "map"]),
        rows(&[
            ("map", "1"),
            ("recip_rank", "1"),
            ("map", "2"),
            ("recip_rank", "2"),
            ("map", "all"),
            ("recip_rank", "all"),
        ])
    );
}

#[test]
fn measures_that_trec_eval_lacks_print_last() {
    // trec_eval refuses ndcg_p, so te-rust prints it after the trec_eval measures.
    assert_eq!(
        printed_rows(&["-m", "ndcg_p", "-m", "map"]),
        rows(&[("map", "all"), ("ndcg_p", "all")])
    );
}

#[test]
fn group_order_is_unchanged() {
    assert_eq!(
        printed_rows(&["-m", "official"])
            .into_iter()
            .take(7)
            .collect::<Vec<_>>(),
        rows(&[
            ("runid", "all"),
            ("num_q", "all"),
            ("num_ret", "all"),
            ("num_rel", "all"),
            ("num_rel_ret", "all"),
            ("map", "all"),
            ("gm_map", "all"),
        ])
    );
}
