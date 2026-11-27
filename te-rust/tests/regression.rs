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

fn parse_ordered_rows(output_str: &str) -> Vec<(String, String, String)> {
    output_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                Some((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn test_regression_unsorted_cutoffs() {
    let qrels_path = get_test_file_path("qrels.test");
    let results_path = get_test_file_path("results.test");
    let qrels_jg_path = get_test_file_path("qrels.123");

    let cases: &[(&[&str], &PathBuf)] = &[
        (&["-m", "map_cut.20,5"], &qrels_path),
        (&["-m", "ndcg_cut.20,5"], &qrels_path),
        (&["-m", "relative_P.20,5"], &qrels_path),
        (&["-m", "unj.20,5"], &qrels_path),
        (&["-m", "P.20,5"], &qrels_path),
        (&["-m", "recall.20,5"], &qrels_path),
        (&["-m", "success.10,1"], &qrels_path),
        (&["-m", "iprec_at_recall.0.8,0.2"], &qrels_path),
        (&["-m", "Rprec_mult.2.0,0.5"], &qrels_path),
        (&["-R", "qrels_jg", "-m", "P_avgjg.20,5"], &qrels_jg_path),
        (&["-R", "qrels_jg", "-m", "Rprec_mult_avgjg.2.0,0.5"], &qrels_jg_path),
    ];

    for (args, qrels) in cases {
        let rust_out = Command::new(get_te_rust_bin())
            .args(*args)
            .arg(qrels.to_str().unwrap())
            .arg(results_path.to_str().unwrap())
            .output()
            .expect("Failed to run te-rust");
        assert!(
            rust_out.status.success(),
            "te-rust failed for {:?}: {}",
            args,
            String::from_utf8_lossy(&rust_out.stderr)
        );

        let c_out = Command::new(get_trec_eval_bin())
            .args(*args)
            .arg(qrels.to_str().unwrap())
            .arg(results_path.to_str().unwrap())
            .output()
            .expect("Failed to run trec_eval");
        assert!(
            c_out.status.success(),
            "trec_eval failed for {:?}: {}",
            args,
            String::from_utf8_lossy(&c_out.stderr)
        );

        let rust_rows = parse_ordered_rows(&String::from_utf8_lossy(&rust_out.stdout));
        let c_rows = parse_ordered_rows(&String::from_utf8_lossy(&c_out.stdout));
        assert_eq!(rust_rows.len(), c_rows.len(), "Row count mismatch for {:?}", args);
        for (r, c) in rust_rows.iter().zip(c_rows.iter()) {
            assert_eq!(r.0, c.0, "Metric name/order mismatch for {:?}", args);
            assert_eq!(r.1, c.1, "QID mismatch for {:?}", args);
            let rf: f64 = r.2.parse().unwrap();
            let cf: f64 = c.2.parse().unwrap();
            assert!(
                (rf - cf).abs() < 1e-4,
                "Value mismatch for {:?} ({}): rust={}, c={}",
                args,
                r.0,
                rf,
                cf
            );
        }
    }

    // 11pt_avg intentionally sorts recall levels (fixing C trec_eval #142):
    // verify descending order produces the same score as ascending order.
    let asc = Command::new(get_te_rust_bin())
        .args(["-m", "11pt_avg.0.2,0.5,0.8"])
        .arg(qrels_path.to_str().unwrap())
        .arg(results_path.to_str().unwrap())
        .output()
        .expect("Failed to run te-rust");
    let desc = Command::new(get_te_rust_bin())
        .args(["-m", "11pt_avg.0.8,0.2,0.5"])
        .arg(qrels_path.to_str().unwrap())
        .arg(results_path.to_str().unwrap())
        .output()
        .expect("Failed to run te-rust");
    assert!(asc.status.success() && desc.status.success());
    let asc_rows = parse_ordered_rows(&String::from_utf8_lossy(&asc.stdout));
    let desc_rows = parse_ordered_rows(&String::from_utf8_lossy(&desc.stdout));
    assert_eq!(asc_rows[0].2, desc_rows[0].2);
}

#[test]
fn test_regression_parameter_validation_and_exit_codes() {
    let qrels_path = get_test_file_path("qrels.test");
    let results_path = get_test_file_path("results.test");
    let rust_bin = get_te_rust_bin();

    // Exit 1: spec / CLI option errors
    let exit_1_cases: &[&[&str]] = &[
        &["-m", "P."],
        &["-m", "P_"],
        &["-m", "P@"],
        &["-m", "ndcg."],
        &["-m", "unknown_measure"],
        &["--global-gains", "1=x", "-m", "ndcg"],
    ];
    for args in exit_1_cases {
        let out = Command::new(&rust_bin)
            .args(*args)
            .arg(qrels_path.to_str().unwrap())
            .arg(results_path.to_str().unwrap())
            .output()
            .expect("Failed to run te-rust");
        assert_eq!(
            out.status.code(),
            Some(1),
            "Expected exit 1 for {:?}, got {:?}; stderr={}",
            args,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // Exit 2: invalid parameter values for known measures
    let exit_2_cases: &[&str] = &[
        "ndcg.1=x",
        "ndcg.x=1",
        "ndcg.5,5",
        "ndcg.1=2,1=5",
        "rbp.p=0.5,q=1",
        "rbp.p=0.5,p=0.8",
        "rbp_resid.x=1",
        "rbp_resid.1=2",
        "P.0",
        "ndcg_cut.0",
        "P.5,5",
        "iprec_at_recall.0.5,0.5",
        "set_F.nan",
        "set_F.inf",
        "rbp.p=nan",
        "utility.1,nan,0,0",
        "iprec_at_recall.1e400",
        "P.9223372036854775808",
        "iprec_at_recall.-1,1.5",
        "11pt_avg.-1,1.5",
        "Rprec_mult.0",
        "Rprec_mult_avgjg.0",
        "set_F.-0.5",
        "rbp.p=1",
        "rbp.p=-0.5",
        "rbp_resid.p=1.5",
        "map.5",
    ];
    for spec in exit_2_cases {
        let out = Command::new(&rust_bin)
            .args(["-m", spec])
            .arg(qrels_path.to_str().unwrap())
            .arg(results_path.to_str().unwrap())
            .output()
            .expect("Failed to run te-rust");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "Expected exit 2 for '{}', got {:?}; stderr={}",
            spec,
            out.status.code(),
            stderr
        );
        assert!(
            stderr.contains(spec),
            "Expected stderr to name '{}', got '{}'",
            spec,
            stderr
        );
    }

    // Legal edge cases (exit 0 and parity with C trec_eval)
    for spec in ["iprec_at_recall.1.", "iprec_at_recall.0", "set_F.0", "rbp.p=0", "rbp.p=0.5,1=2,2=5"] {
        compare_outputs(&["-m", spec], &["-m", spec], "qrels.test", "results.test");
    }
    compare_outputs(&["-q", "-m", "relstring.0"], &["-q", "-m", "relstring.0"], "qrels.test", "results.test");
}

#[test]
fn test_regression_run_score_validation_issue_107() {
    struct TempDirGuard(PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let dir = std::env::temp_dir().join(format!(
        "te_rust_issue_107_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    let _guard = TempDirGuard(dir.clone());

    let qrels_path = dir.join("abc.qrels");
    std::fs::write(&qrels_path, "1 0 a 1\n1 0 b 2\n1 0 c 3\n").unwrap();

    let ok_run = dir.join("ok.run");
    std::fs::write(&ok_run, "1 Q0 a 1 5 r\n1 Q0 b 2 4 r\n1 Q0 c 3 3 r\n").unwrap();

    let rust_bin = get_te_rust_bin();
    let ok_out = Command::new(&rust_bin)
        .args(["-q", "-m", "relstring"])
        .arg(&qrels_path)
        .arg(&ok_run)
        .output()
        .expect("Failed to run te-rust");
    assert!(
        ok_out.status.success(),
        "expected ok.run to succeed, stderr={}",
        String::from_utf8_lossy(&ok_out.stderr)
    );
    let ok_rows = parse_ordered_rows(&String::from_utf8_lossy(&ok_out.stdout));
    assert_eq!(
        ok_rows,
        vec![("relstring".to_string(), "1".to_string(), "'123'".to_string())]
    );

    // Issue #107 reproduction cases plus inf/-inf/overflow (strictly rejected by te-rust)
    for (name, bad_score) in [
        ("nan.run", "nan"),
        ("abc.run", "abc"),
        ("trail.run", "1.5x"),
        ("inf.run", "inf"),
        ("neginf.run", "-inf"),
        ("overflow.run", "1e400"),
    ] {
        let run_path = dir.join(name);
        std::fs::write(
            &run_path,
            format!("1 Q0 a 1 5 r\n1 Q0 b 2 {} r\n1 Q0 c 3 3 r\n", bad_score),
        )
        .unwrap();

        let out = Command::new(&rust_bin)
            .args(["-q", "-m", "relstring"])
            .arg(&qrels_path)
            .arg(&run_path)
            .output()
            .expect("Failed to run te-rust");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(1),
            "expected exit 1 for {} (score '{}'), got {:?}; stdout={}, stderr={}",
            name,
            bad_score,
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            stderr
        );
        assert!(
            stderr.contains("Line 2") && stderr.contains(bad_score),
            "expected stderr for {} to cite line 2 and '{}', got '{}'",
            name,
            bad_score,
            stderr
        );
    }
}

/// Port of C PR #205's `test/test_rel_levels.sh`: verifies sparse distinct
/// relevance-level handling across #94, #97, #98, #102, #142 item 9, and
/// #143 item 4 directly on `te-rust`.
#[test]
fn test_regression_rel_levels_and_issue_102() {
    struct TempDirGuard(PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let dir = std::env::temp_dir().join(format!(
        "te_rust_rel_levels_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    let _guard = TempDirGuard(dir.clone());
    let rust_bin = get_te_rust_bin();

    let run_eval = |desc: &str, args: &[&str], qrels: &PathBuf, run: &PathBuf| -> HashMap<(String, String), String> {
        let out = Command::new(&rust_bin)
            .args(args)
            .arg(qrels)
            .arg(run)
            .output()
            .expect("Failed to run te-rust");
        assert!(
            out.status.success(),
            "{}: expected exit 0, got {:?}; stderr={}",
            desc,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut map = HashMap::new();
        for (m, q, v) in parse_ordered_rows(&stdout) {
            let lower = v.to_ascii_lowercase();
            assert!(
                !lower.contains("nan") && !lower.contains("inf"),
                "{}: unexpected NaN/Inf in '{} {} {}'",
                desc,
                m,
                q,
                v
            );
            map.insert((m, q), v);
        }
        map
    };

    let assert_val = |map: &HashMap<(String, String), String>, desc: &str, m: &str, q: &str, expected: &str| {
        let actual = map.get(&(m.to_string(), q.to_string())).map(|s| s.as_str());
        assert_eq!(
            actual,
            Some(expected),
            "{}: {} {} expected '{}', got {:?}",
            desc,
            m,
            q,
            expected,
            actual
        );
    };

    // --- Issue #102: bpref / gm_bpref (and rbp) when -l exceeds highest grade or is negative ---
    let bpref_qrels = dir.join("bpref.qrels");
    let bpref_run = dir.join("bpref.run");
    std::fs::write(&bpref_qrels, "1 0 a 2\n1 0 b 0\n1 0 c 1\n1 0 d 2\n2 0 a 1\n2 0 b 0\n").unwrap();
    std::fs::write(
        &bpref_run,
        "1 Q0 a 1 4 r\n1 Q0 c 2 3 r\n1 Q0 b 3 2 r\n1 Q0 d 4 1 r\n2 Q0 a 1 2 r\n2 Q0 b 2 1 r\n",
    )
    .unwrap();

    let out = run_eval("issue102 -l 1", &["-q", "-l", "1", "-m", "bpref", "-m", "gm_bpref"], &bpref_qrels, &bpref_run);
    assert_val(&out, "issue102 -l 1", "bpref", "1", "0.6667");
    assert_val(&out, "issue102 -l 1", "bpref", "2", "1.0000");
    assert_val(&out, "issue102 -l 1", "bpref", "all", "0.8333");

    let out = run_eval("issue102 -l 4", &["-q", "-l", "4", "-m", "bpref", "-m", "gm_bpref", "-m", "rbp"], &bpref_qrels, &bpref_run);
    assert_val(&out, "issue102 -l 4", "bpref", "1", "0.0000");
    assert_val(&out, "issue102 -l 4", "bpref", "2", "0.0000");
    assert_val(&out, "issue102 -l 4", "bpref", "all", "0.0000");
    assert_val(&out, "issue102 -l 4", "gm_bpref", "all", "0.0000");
    assert_val(&out, "issue102 -l 4", "rbp", "2", "0.1000");

    for huge_l in ["1000000000000", "9223372036854775807"] {
        let out = run_eval("issue102 huge -l", &["-l", huge_l, "-m", "bpref", "-m", "gm_bpref"], &bpref_qrels, &bpref_run);
        assert_val(&out, "issue102 huge -l", "bpref", "all", "0.0000");
        assert_val(&out, "issue102 huge -l", "gm_bpref", "all", "0.0000");
    }

    let out = run_eval("issue102 negative -l", &["-q", "-l", "-1", "-m", "bpref", "-m", "gm_bpref"], &bpref_qrels, &bpref_run);
    assert_val(&out, "issue102 negative -l", "bpref", "1", "1.0000");
    assert_val(&out, "issue102 negative -l", "bpref", "2", "1.0000");
    assert_val(&out, "issue102 negative -l", "bpref", "all", "1.0000");
    assert_val(&out, "issue102 negative -l", "gm_bpref", "all", "1.0000");

    // --- Issue #97: topics whose qrels grades are all negative ---
    let neg_first_q = dir.join("neg-first.qrels");
    let neg_first_r = dir.join("neg-first.run");
    let neg_second_q = dir.join("neg-second.qrels");
    let neg2_second_q = dir.join("neg2-second.qrels");
    let neg_second_r = dir.join("neg-second.run");
    std::fs::write(&neg_first_q, "1 0 d1 -1\n1 0 d2 -1\n2 0 d1 0\n2 0 d2 1\n").unwrap();
    std::fs::write(&neg_first_r, "1 Q0 d1 1 2 r\n1 Q0 d2 2 1 r\n2 Q0 d2 1 2 r\n2 Q0 d1 2 1 r\n").unwrap();
    std::fs::write(&neg_second_q, "0 0 d1 0\n0 0 d2 1\n1 0 d1 -1\n1 0 d2 -1\n").unwrap();
    std::fs::write(&neg2_second_q, "0 0 d1 0\n0 0 d2 1\n1 0 d1 -2\n1 0 d2 -2\n").unwrap();
    std::fs::write(&neg_second_r, "0 Q0 d2 1 2 r\n0 Q0 d1 2 1 r\n1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n").unwrap();

    let out = run_eval("neg-first map", &["-q", "-m", "map"], &neg_first_q, &neg_first_r);
    assert_val(&out, "neg-first map", "map", "1", "0.0000");
    assert_val(&out, "neg-first map", "map", "2", "1.0000");
    assert_val(&out, "neg-first map", "map", "all", "0.5000");

    for (label, qfile) in [("neg-second", &neg_second_q), ("neg2-second", &neg2_second_q)] {
        let out = run_eval(label, &["-q", "-m", "map"], qfile, &neg_second_r);
        assert_val(&out, label, "map", "0", "1.0000");
        assert_val(&out, label, "map", "1", "0.0000");
        assert_val(&out, label, "map", "all", "0.5000");
    }

    for m in ["ndcg", "ndcg_rel", "G"] {
        let spec = format!("{}.-2=1", m);
        let key = format!("{}_-2=1", m);
        let out = run_eval(&spec, &["-q", "-m", &spec], &neg_second_q, &neg_second_r);
        assert_val(&out, &spec, &key, "0", "1.0000");
        assert_val(&out, &spec, &key, "1", "0.0000");
        assert_val(&out, &spec, &key, "all", "0.5000");
    }

    let out = run_eval("neg-first ndcg_cut huge cutoff", &["-q", "-m", "ndcg_cut.2147483647"], &neg_first_q, &neg_first_r);
    assert_val(&out, "neg-first ndcg_cut", "ndcg_cut_2147483647", "1", "0.0000");
    assert_val(&out, "neg-first ndcg_cut", "ndcg_cut_2147483647", "2", "1.0000");
    assert_val(&out, "neg-first ndcg_cut", "ndcg_cut_2147483647", "all", "0.5000");

    let out = run_eval("neg-first all_trec", &["-q", "-m", "all_trec"], &neg_first_q, &neg_first_r);
    assert_val(&out, "neg-first all_trec", "ndcg", "1", "0.0000");
    assert_val(&out, "neg-first all_trec", "rbp", "1", "0.0000");

    let out = run_eval("neg-second negative gain override", &["-q", "-m", "ndcg.0=-1,1=2"], &neg_second_q, &neg_second_r);
    assert_val(&out, "negative gain override", "ndcg_0=-1,1=2", "0", "0.6845");
    assert_val(&out, "negative gain override", "ndcg_0=-1,1=2", "1", "0.0000");
    assert_val(&out, "negative gain override", "ndcg_0=-1,1=2", "all", "0.3423");

    // --- Issue #143 item 4: sparse/large qrels grades including i64::MAX ---
    let huge_q = dir.join("huge.qrels");
    let huge_r = dir.join("huge.run");
    std::fs::write(&huge_q, "1 0 d1 0\n1 0 d2 1000000000\n1 0 d3 500000000\n").unwrap();
    std::fs::write(&huge_r, "1 Q0 d2 1 3 r\n1 Q0 d3 2 2 r\n1 Q0 d1 3 1 r\n").unwrap();

    let out = run_eval(
        "sparse huge grades",
        &["-q", "-m", "map", "-m", "ndcg", "-m", "ndcg_cut.5", "-m", "bpref", "-m", "gm_bpref"],
        &huge_q,
        &huge_r,
    );
    assert_val(&out, "sparse huge grades", "map", "1", "1.0000");
    assert_val(&out, "sparse huge grades", "ndcg", "1", "1.0000");
    assert_val(&out, "sparse huge grades", "ndcg_cut_5", "1", "1.0000");
    assert_val(&out, "sparse huge grades", "bpref", "1", "1.0000");
    assert_val(&out, "sparse huge grades", "gm_bpref", "all", "1.0000");

    let max_q = dir.join("max.qrels");
    let two_r = dir.join("two.run");
    std::fs::write(&max_q, "1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n2 0 d1 9223372036854775807\n").unwrap();
    std::fs::write(&two_r, "1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n2 Q0 d1 1 1 r\n2 Q0 d2 2 0 r\n").unwrap();

    let out = run_eval("i64::MAX map", &["-m", "map"], &max_q, &two_r);
    assert_val(&out, "i64::MAX map", "map", "all", "0.9167");

    let out = run_eval("i64::MAX qrels_jg", &["-R", "qrels_jg", "-m", "map_avgjg"], &max_q, &two_r);
    assert_val(&out, "i64::MAX qrels_jg", "map_avgjg", "all", "0.9167");

    let jg_q = dir.join("jg.qrels");
    std::fs::write(&jg_q, "0 jg1 d1 0\n0 jg1 d2 1\n1 jg1 d1 -2\n1 jg1 d2 -2\n1 jg2 d1 100000000\n1 jg2 d2 0\n").unwrap();
    let out = run_eval("qrels_jg neg and sparse", &["-q", "-R", "qrels_jg", "-m", "qrels_jg"], &jg_q, &neg_second_r);
    assert_val(&out, "qrels_jg neg and sparse", "map_avgjg", "0", "1.0000");
    assert_val(&out, "qrels_jg neg and sparse", "map_avgjg", "1", "0.5000");
    assert_val(&out, "qrels_jg neg and sparse", "map_avgjg", "all", "0.7500");

    // --- Issue #98: fractional gains < 1 apart and ndcg_rel zero ideal_dcg ---
    let gains_q = dir.join("gains.qrels");
    let gains_r = dir.join("gains.run");
    std::fs::write(&gains_q, "1 0 d1 1\n1 0 d2 2\n1 0 d3 3\n").unwrap();
    std::fs::write(&gains_r, "1 Q0 d2 1 3 r\n1 Q0 d3 2 2 r\n1 Q0 d1 3 1 r\n").unwrap();

    let out = run_eval("issue98 scaled gains", &["-m", "ndcg.1=0.3,2=0.6,3=0.9"], &gains_q, &gains_r);
    assert_val(&out, "issue98 scaled gains", "ndcg_1=0.3,2=0.6,3=0.9", "all", "0.9225");

    for (m, key) in [
        ("ndcg.0=0,1=0.3,2=0.9,3=0.6", "ndcg_0=0,1=0.3,2=0.9,3=0.6"),
        ("Rndcg.0=0,1=0.3,2=0.9,3=0.6", "Rndcg_0=0,1=0.3,2=0.9,3=0.6"),
        ("ndcg_rel.0=0,1=0.3,2=0.9,3=0.6", "ndcg_rel_0=0,1=0.3,2=0.9,3=0.6"),
    ] {
        let out = run_eval(m, &["-m", m], &gains_q, &gains_r);
        assert_val(&out, m, key, "all", "1.0000");
    }

    let neg_gain_q = dir.join("neg_gain.qrels");
    let neg_gain_r = dir.join("neg_gain.run");
    std::fs::write(&neg_gain_q, "1 0 n -2\n1 0 a 2\n").unwrap();
    std::fs::write(&neg_gain_r, "1 Q0 n 1 2 r\n1 Q0 a 2 1 r\n").unwrap();
    let out = run_eval("issue98 ndcg_rel zero ideal", &["-q", "-l", "2", "-m", "ndcg_rel.-2=1,2=-3"], &neg_gain_q, &neg_gain_r);
    assert_val(&out, "issue98 ndcg_rel zero ideal", "ndcg_rel_-2=1,2=-3", "1", "0.0000");
    assert_val(&out, "issue98 ndcg_rel zero ideal", "ndcg_rel_-2=1,2=-3", "all", "0.0000");

    // --- Issue #142 item 9: Rndcg with num_rel > 0 and no positive gain ---
    let five_q = dir.join("five.qrels");
    let five_r = dir.join("five.run");
    std::fs::write(&five_q, "1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n2 0 d1 0\n2 0 d2 1\n").unwrap();
    std::fs::write(&five_r, "1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n2 Q0 d2 1 2 r\n2 Q0 d1 2 1 r\n").unwrap();
    let out = run_eval("issue142 Rndcg.1=0", &["-q", "-m", "Rndcg.1=0"], &five_q, &five_r);
    assert_val(&out, "issue142 Rndcg.1=0", "Rndcg_1=0", "2", "0.0000");
    assert_val(&out, "issue142 Rndcg.1=0", "Rndcg_1=0", "all", "0.1250");

    let allzero_q = dir.join("allzero.qrels");
    let allzero_r = dir.join("allzero.run");
    std::fs::write(&allzero_q, "1 0 a 0\n1 0 b 0\n").unwrap();
    std::fs::write(&allzero_r, "1 Q0 a 1 2 r\n1 Q0 b 2 1 r\n").unwrap();
    let out = run_eval("issue142 -l 0 Rndcg all-zero", &["-l", "0", "-q", "-m", "Rndcg"], &allzero_q, &allzero_r);
    assert_val(&out, "issue142 -l 0 Rndcg all-zero", "Rndcg", "1", "0.0000");
    assert_val(&out, "issue142 -l 0 Rndcg all-zero", "Rndcg", "all", "0.0000");

    // --- Issue #94: rbp rescaled from 0.0 ---
    let rbp_q = dir.join("rbp.qrels");
    let rbp_r = dir.join("rbp.run");
    std::fs::write(&rbp_q, "1 0 a 1\n1 0 b 0\n").unwrap();
    std::fs::write(&rbp_r, "1 Q0 a 1 2 r\n1 Q0 b 2 1 r\n").unwrap();
    let out = run_eval("issue94 rbp rescaled from 0", &["-m", "rbp.0=1,1=3"], &rbp_q, &rbp_r);
    assert_val(&out, "issue94 rbp rescaled from 0", "rbp_0=1,1=3", "all", "0.1300");
}

