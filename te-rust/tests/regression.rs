use std::collections::HashMap;
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

fn parse_output(output_str: &str) -> HashMap<(String, String), Val> {
    let mut results = HashMap::new();
    for line in output_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let measure = parts[0].to_string();
        let qid = parts[1].to_string();
        let val_str = parts[2];

        let val = if let Ok(i) = val_str.parse::<i64>() {
            Val::Int(i)
        } else if let Ok(f) = val_str.parse::<f64>() {
            Val::Float(f)
        } else {
            Val::Str(val_str.to_string())
        };

        results.insert((measure, qid), val);
    }
    results
}

fn compare_outputs(
    rust_args: &[&str],
    c_args: &[&str],
    qrels_name: &str,
    results_name: &str,
) {
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
    let rust_map = parse_output(&rust_str);

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
    let c_map = parse_output(&c_str);

    // 3. Compare common keys
    let epsilon = 1e-4;
    let mut matched_count = 0;

    for (key, c_val) in &c_map {
        // Since we only implement all_trec standard measures, we check if rust_map contains it
        if let Some(rust_val) = rust_map.get(key) {
            matched_count += 1;
            match (c_val, rust_val) {
                (Val::Float(c_f), Val::Float(r_f)) => {
                    assert!(
                        (c_f - r_f).abs() < epsilon,
                        "Mismatch for {:?}: C value {}, Rust value {}",
                        key,
                        c_f,
                        r_f
                    );
                }
                (Val::Int(c_i), Val::Int(r_i)) => {
                    assert_eq!(
                        c_i, r_i,
                        "Mismatch for {:?}: C value {}, Rust value {}",
                        key, c_i, r_i
                    );
                }
                (Val::Str(c_s), Val::Str(r_s)) => {
                    assert_eq!(
                        c_s, r_s,
                        "Mismatch for {:?}: C value '{}', Rust value '{}'",
                        key, c_s, r_s
                    );
                }
                _ => {
                    panic!(
                        "Type mismatch for {:?}: C value {:?}, Rust value {:?}",
                        key, c_val, rust_val
                    );
                }
            }
        }
    }

    assert!(
        matched_count > 0,
        "No matching metrics found between C and Rust output!"
    );
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

