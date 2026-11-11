use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cutoff-order")
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

/// The value at a cutoff cannot depend on which other cutoffs are listed, or in what order.
fn assert_order_free(extra: &[&str], qrels: &str, run: &str, measure: &str, low: &str, high: &str) {
    let (qrels, run) = (fixture(qrels), fixture(run));
    let eval = |spec: String| {
        let mut args: Vec<&str> = extra.to_vec();
        args.extend(["-m", spec.as_str(), qrels.as_str(), run.as_str()]);
        rows(&te_rust(&args))
    };
    let mut alone = eval(format!("{measure}.{low}"));
    alone.extend(eval(format!("{measure}.{high}")));
    assert_eq!(
        eval(format!("{measure}.{low},{high}")),
        alone,
        "{measure} ascending"
    );
    assert_eq!(
        eval(format!("{measure}.{high},{low}")),
        alone,
        "{measure} descending"
    );
}

#[test]
fn every_cutoff_family_is_order_free() {
    for measure in [
        "P",
        "recall",
        "success",
        "map_cut",
        "ndcg_cut",
        "relative_P",
        "unj",
    ] {
        assert_order_free(&[], "mixed.qrels", "mixed.run", measure, "2", "5");
    }
    assert_order_free(
        &[],
        "mixed.qrels",
        "mixed.run",
        "iprec_at_recall",
        "0.2",
        "0.8",
    );
    assert_order_free(&[], "mixed.qrels", "mixed.run", "Rprec_mult", "0.5", "2.0");
    let jg = ["-R", "qrels_jg"];
    assert_order_free(&jg, "mixed.qrels_jg", "mixed.run", "P_avgjg", "2", "5");
    assert_order_free(
        &jg,
        "mixed.qrels_jg",
        "mixed.run",
        "Rprec_mult_avgjg",
        "0.5",
        "2.0",
    );
}

#[test]
fn map_cut_matches_hand_calculation_when_descending() {
    // Six relevant documents, all retrieved in the top six.
    // map_cut_2 = (1 + 1) / 6, map_cut_5 = (1 + 1 + 1 + 1 + 1) / 6.
    let out = te_rust(&[
        "-m",
        "map_cut.5,2",
        &fixture("six-relevant.qrels"),
        &fixture("six-relevant.run"),
    ]);
    assert_eq!(summary(&out, "map_cut_2"), "0.3333");
    assert_eq!(summary(&out, "map_cut_5"), "0.8333");
}

#[test]
fn relative_p_cannot_exceed_one() {
    let out = te_rust(&[
        "-m",
        "relative_P.5,2",
        &fixture("six-relevant.qrels"),
        &fixture("six-relevant.run"),
    ]);
    assert_eq!(summary(&out, "relative_P_2"), "1.0000");
    assert_eq!(summary(&out, "relative_P_5"), "1.0000");
}

#[test]
fn rprec_mult_reaches_every_multiple_when_descending() {
    // R = 5. Rprec_mult_2.00 is precision at rank 10: a, c, e and f are relevant, 4 / 10.
    let out = te_rust(&[
        "-m",
        "Rprec_mult.2.0,0.5",
        &fixture("mixed.qrels"),
        &fixture("mixed.run"),
    ]);
    assert_eq!(summary(&out, "Rprec_mult_2.00"), "0.4000");
}

#[test]
fn rows_print_in_ascending_cutoff_order_like_trec_eval() {
    let out = te_rust(&[
        "-m",
        "P.5,2",
        &fixture("mixed.qrels"),
        &fixture("mixed.run"),
    ]);
    let names: Vec<String> = rows(&out).into_iter().map(|row| row.0).collect();
    assert_eq!(names, ["P_2", "P_5"]);
}

#[test]
fn utility_coefficients_keep_their_positions() {
    // rel_ret * 2 - nonrel_ret * 1 = 6 * 2. Sorting the list would give -18.
    let out = te_rust(&[
        "-m",
        "utility.2,-1,0,0",
        &fixture("six-relevant.qrels"),
        &fixture("six-relevant.run"),
    ]);
    assert_eq!(summary(&out, "utility_2,-1,0,0"), "12.0000");
}

#[test]
fn eleven_pt_avg_keeps_list_order_like_trec_eval() {
    // trec_eval reads 11pt_avg with get_double_params, which does not sort,
    // and walks the list from its last entry, so both tools depend on the order.
    let qrels = fixture("two-relevant.qrels");
    let run = fixture("two-relevant.run");
    let ascending = te_rust(&["-m", "11pt_avg.0.5,1.0", &qrels, &run]);
    let descending = te_rust(&["-m", "11pt_avg.1.0,0.5", &qrels, &run]);
    assert_eq!(rows(&ascending)[0].2, "0.7500");
    assert_eq!(rows(&descending)[0].2, "1.0000");
}
