use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;

fn get_te_rust_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_te-rust"))
}

fn get_trec_eval_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go to workspace root
    path.push(get_trec_eval_dir_name());
    path.push("trec_eval");
    path
}

fn get_test_file_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go to workspace root
    path.push(get_trec_eval_dir_name());
    path.push("test");
    path.push(filename);
    path
}

fn get_trec_eval_dir_name() -> String {
    std::env::var("TREC_EVAL_DIR").unwrap_or_else(|_| "trec_eval".to_string())
}

#[derive(Debug, Clone, PartialEq)]
enum Val {
    Float(f64),
    Int(i64),
    Str(String),
}

type Row = (String, String, Val);

/// The printed rows as (measure, topic, value), in output order.
fn parse_output(output_str: &str) -> Vec<Row> {
    output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let [measure, qid, val_str] = parts[..] else {
                panic!("output line without three fields: {line:?}");
            };
            let val = if let Ok(i) = val_str.parse::<i64>() {
                Val::Int(i)
            } else if let Ok(f) = val_str.parse::<f64>() {
                Val::Float(f)
            } else {
                Val::Str(val_str.to_string())
            };
            (measure.to_string(), qid.to_string(), val)
        })
        .collect()
}

fn values_match(c_val: &Val, rust_val: &Val) -> bool {
    match (c_val, rust_val) {
        (Val::Float(c_f), Val::Float(r_f)) => (c_f - r_f).abs() < 1e-4,
        _ => c_val == rust_val,
    }
}

/// Every difference between the two outputs, with C as the reference.
fn output_differences(c_rows: &[Row], rust_rows: &[Row]) -> Vec<String> {
    let key = |(measure, qid, _): &Row| (measure.clone(), qid.clone());
    let mut differences = Vec::new();
    for (label, rows) in [("C", c_rows), ("te-rust", rust_rows)] {
        let mut counts: HashMap<(String, String), usize> = HashMap::new();
        for row in rows {
            *counts.entry(key(row)).or_default() += 1;
        }
        let mut repeated: Vec<_> = counts.into_iter().filter(|(_, n)| *n > 1).collect();
        repeated.sort();
        for ((measure, qid), n) in repeated {
            differences.push(format!("{label} prints {measure} {qid} {n} times"));
        }
    }
    let c_keys: HashSet<_> = c_rows.iter().map(key).collect();
    let rust_keys: HashSet<_> = rust_rows.iter().map(key).collect();
    for (measure, qid, _) in c_rows.iter().filter(|row| !rust_keys.contains(&key(row))) {
        differences.push(format!("te-rust lacks {measure} {qid}"));
    }
    for (measure, qid, _) in rust_rows.iter().filter(|row| !c_keys.contains(&key(row))) {
        differences.push(format!("te-rust adds {measure} {qid}"));
    }
    let rust_values: HashMap<_, _> = rust_rows.iter().map(|row| (key(row), &row.2)).collect();
    for row in c_rows {
        if let Some(rust_val) = rust_values.get(&key(row)) {
            if !values_match(&row.2, rust_val) {
                differences.push(format!(
                    "{} {}: C {:?}, te-rust {:?}",
                    row.0, row.1, row.2, rust_val
                ));
            }
        }
    }
    // The order of the rows that both tools print.
    let c_order: Vec<_> = c_rows
        .iter()
        .map(key)
        .filter(|k| rust_keys.contains(k))
        .collect();
    let rust_order: Vec<_> = rust_rows
        .iter()
        .map(key)
        .filter(|k| c_keys.contains(k))
        .collect();
    if let Some(i) = c_order.iter().zip(&rust_order).position(|(c, r)| c != r) {
        differences.push(format!(
            "row order differs at shared row {}: C {:?}, te-rust {:?}",
            i + 1,
            c_order[i],
            rust_order[i]
        ));
    }
    differences
}

fn compare_outputs(rust_args: &[&str], c_args: &[&str], qrels_name: &str, results_name: &str) {
    let qrels_path = get_test_file_path(qrels_name);
    let results_path = get_test_file_path(results_name);

    // 1. Run te-rust
    let rust_bin = get_te_rust_bin();
    let mut rust_cmd = Command::new(&rust_bin);
    for arg in rust_args {
        rust_cmd.arg(arg);
    }
    rust_cmd.arg(qrels_path.to_str().unwrap());
    rust_cmd.arg(results_path.to_str().unwrap());

    let rust_output = rust_cmd.output().expect("Failed to execute te-rust");
    assert!(
        rust_output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
    let rust_str = String::from_utf8_lossy(&rust_output.stdout);
    let rust_rows = parse_output(&rust_str);

    // 2. Run C trec_eval
    let c_bin = get_trec_eval_bin();
    let mut c_cmd = Command::new(&c_bin);
    for arg in c_args {
        c_cmd.arg(arg);
    }
    c_cmd.arg(qrels_path.to_str().unwrap());
    c_cmd.arg(results_path.to_str().unwrap());

    let c_output = c_cmd.output().expect("Failed to execute C trec_eval");
    assert!(
        c_output.status.success(),
        "C trec_eval failed: {}",
        String::from_utf8_lossy(&c_output.stderr)
    );
    let c_str = String::from_utf8_lossy(&c_output.stdout);
    let c_rows = parse_output(&c_str);

    // 3. Both tools print the same rows in the same order, with values within 1e-4.
    assert!(!c_rows.is_empty(), "C trec_eval printed no rows");
    let differences = output_differences(&c_rows, &rust_rows);
    assert!(
        differences.is_empty(),
        "te-rust {:?} differs from C trec_eval {:?}:\n{}",
        rust_args,
        c_args,
        differences.join("\n")
    );
}

#[test]
fn harness_reports_every_kind_of_difference() {
    let c = parse_output("num_q\tall\t2\nmap\tall\t0.5000\nP_5\tall\t0.2000\n");
    assert!(output_differences(&c, &c).is_empty());
    let within_tolerance = parse_output("num_q\tall\t2\nmap\tall\t0.50004\nP_5\tall\t0.2000\n");
    assert!(output_differences(&c, &within_tolerance).is_empty());

    let cases = [
        ("num_q\tall\t2\nmap\tall\t0.5000\n", "te-rust lacks P_5 all"),
        (
            "num_q\tall\t2\nmap\tall\t0.5000\nmap\tall\t0.5000\nP_5\tall\t0.2000\n",
            "te-rust prints map all 2 times",
        ),
        (
            "num_q\tall\t2\nmap\tall\t0.5000\nP_5\tall\t0.2000\nP_10\tall\t0.1000\n",
            "te-rust adds P_10 all",
        ),
        (
            "map\tall\t0.5000\nnum_q\tall\t2\nP_5\tall\t0.2000\n",
            "row order differs at shared row 1",
        ),
        (
            "num_q\tall\t2\nmap\tall\t0.5002\nP_5\tall\t0.2000\n",
            "map all: C Float(0.5), te-rust Float(0.5002)",
        ),
        (
            "num_q\tall\t2.0000\nmap\tall\t0.5000\nP_5\tall\t0.2000\n",
            "num_q all: C Int(2), te-rust Float(2.0)",
        ),
    ];
    for (rust, expected) in cases {
        let differences = output_differences(&c, &parse_output(rust));
        assert!(
            differences.iter().any(|d| d.starts_with(expected)),
            "{expected:?} not in {differences:?}"
        );
    }
}

#[test]
fn test_regression_default() {
    compare_outputs(&[], &[], "qrels.test", "results.test");
}

#[test]
fn test_regression_query_level() {
    compare_outputs(&["-q"], &["-q"], "qrels.test", "results.test");
}

#[test]
fn test_regression_judged_only() {
    compare_outputs(&["-J"], &["-J"], "qrels.test", "results.test");
}

#[test]
fn test_regression_max_docs() {
    compare_outputs(&["-M", "100"], &["-M", "100"], "qrels.test", "results.test");
}

#[test]
fn test_regression_relevance_level() {
    compare_outputs(&["-l", "2"], &["-l", "2"], "qrels.test", "results.test");
}

#[test]
fn test_regression_parametric_cutoffs() {
    compare_outputs(
        &["-m", "P.5", "-m", "ndcg_cut.10"],
        &["-m", "P.5", "-m", "ndcg_cut.10"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_stock_set() {
    compare_outputs(
        &["-m", "set"],
        &["-m", "set"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_help_flags() {
    let rust_bin = get_te_rust_bin();
    
    // Test --help-measures
    let output_measures = Command::new(&rust_bin)
        .arg("--help-measures")
        .output()
        .expect("Failed to run --help-measures");
    assert!(output_measures.status.success());
    let stdout_m = String::from_utf8_lossy(&output_measures.stdout);
    assert!(stdout_m.contains("map"));
    assert!(stdout_m.contains("Mean Average Precision"));

    // Test --help-measure map
    let output_map = Command::new(&rust_bin)
        .arg("--help-measure")
        .arg("map")
        .output()
        .expect("Failed to run --help-measure map");
    assert!(output_map.status.success());
    let stdout_map = String::from_utf8_lossy(&output_map.stdout);
    assert!(stdout_map.contains("Calculates the average of precision scores"));
}

#[test]
fn test_regression_specialized_standard_metrics() {
    compare_outputs(
        &["-m", "recall.5,10", "-m", "success.1,5", "-m", "11pt_avg", "-m", "utility", "-m", "ndcg"],
        &["-m", "recall.5,10", "-m", "success.1,5", "-m", "11pt_avg", "-m", "utility", "-m", "ndcg"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_relstring_query_level() {
    compare_outputs(
        &["-q", "-m", "relstring.10"],
        &["-q", "-m", "relstring.10"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_group_3() {
    compare_outputs(
        &["-m", "map_cut", "-m", "relative_P", "-m", "Rprec_mult", "-m", "iprec_at_recall"],
        &["-m", "map_cut", "-m", "relative_P", "-m", "Rprec_mult", "-m", "iprec_at_recall"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_group_4() {
    compare_outputs(
        &["-q", "-m", "gm_map", "-m", "gm_bpref", "-m", "infAP"],
        &["-q", "-m", "gm_map", "-m", "gm_bpref", "-m", "infAP"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_group_5_7() {
    // 1. Standard cases (includes experimental measures G, binG, yaap, which are
    //    excluded from groups but must still be verified against C trec_eval).
    compare_outputs(
        &["-q", "-m", "unj", "-m", "num_nonrel_judged_ret", "-m", "rbp", "-m", "rbp_resid", "-m", "yaap", "-m", "G", "-m", "binG"],
        &["-q", "-m", "unj", "-m", "num_nonrel_judged_ret", "-m", "rbp", "-m", "rbp_resid", "-m", "yaap", "-m", "G", "-m", "binG"],
        "qrels.test",
        "results.test",
    );

    // 2. Parameterized cases
    compare_outputs(
        &["-q", "-m", "unj.5,15", "-m", "rbp.p=0.95", "-m", "rbp_resid.p=0.95"],
        &["-q", "-m", "unj.5,15", "-m", "rbp.p=0.95", "-m", "rbp_resid.p=0.95"],
        "qrels.test",
        "results.test",
    );
}

#[test]
fn test_regression_qrels_jg() {
    compare_outputs(
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        "qrels.123",
        "results.test",
    );
}

#[test]
fn test_regression_qrels_jg_with_comments() {
    compare_outputs(
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        "qrels.comments.123",
        "results.test",
    );
}

