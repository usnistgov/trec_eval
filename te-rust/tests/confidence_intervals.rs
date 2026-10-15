//! Integration tests for bootstrap confidence interval estimation (ISSUES #22).

use std::path::PathBuf;
use std::process::Command;

fn get_te_rust_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_te-rust"))
}

fn get_test_file_path(filename: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("tests").join("test-data").join(filename)
}

#[test]
fn test_ci_relational_format() {
    let bin = get_te_rust_bin();
    let qrels = get_test_file_path("qrels.test");
    let results = get_test_file_path("results.test");

    let output = Command::new(&bin)
        .args(["-m", "map", "-m", "P.5", "-m", "num_rel", "-m", "runid"])
        .arg(&qrels)
        .arg(&results)
        .args(["-C", "--seed", "42", "--ci-samples", "1000"])
        .output()
        .expect("Failed to run te-rust");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Every line must be strictly 3 columns
    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 3, "Line was not 3 tab-separated columns: '{}'", line);
    }

    let map_line = lines.iter().find(|l| l.starts_with("map ")).unwrap();
    let map_lower = lines.iter().find(|l| l.starts_with("map_ci_lower ")).unwrap();
    let map_upper = lines.iter().find(|l| l.starts_with("map_ci_upper ")).unwrap();

    let map_val: f64 = map_line.split('\t').nth(2).unwrap().parse().unwrap();
    let lower_val: f64 = map_lower.split('\t').nth(2).unwrap().parse().unwrap();
    let upper_val: f64 = map_upper.split('\t').nth(2).unwrap().parse().unwrap();

    assert!(lower_val <= map_val, "lower bound {} > point estimate {}", lower_val, map_val);
    assert!(map_val <= upper_val, "point estimate {} > upper bound {}", map_val, upper_val);

    // Non-averaged measures should NOT have _ci_lower or _ci_upper rows
    assert!(!lines.iter().any(|l| l.starts_with("num_rel_ci_")));
    assert!(!lines.iter().any(|l| l.starts_with("runid_ci_")));
}

#[test]
fn test_ci_pretty_format() {
    let bin = get_te_rust_bin();
    let qrels = get_test_file_path("qrels.test");
    let results = get_test_file_path("results.test");

    let output = Command::new(&bin)
        .args(["-m", "map", "-m", "num_rel"])
        .arg(&qrels)
        .arg(&results)
        .args(["--ci-pretty", "--seed", "42"])
        .output()
        .expect("Failed to run te-rust");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    let map_line = lines.iter().find(|l| l.starts_with("map ")).unwrap();
    assert!(map_line.contains('['), "map line missing bracketed interval: {}", map_line);
    assert!(map_line.contains(']'), "map line missing bracketed interval: {}", map_line);

    let num_rel_line = lines.iter().find(|l| l.starts_with("num_rel ")).unwrap();
    assert!(!num_rel_line.contains('['), "num_rel should not have bracketed interval: {}", num_rel_line);
}

#[test]
fn test_ci_deterministic_seeding() {
    let bin = get_te_rust_bin();
    let qrels = get_test_file_path("qrels.test");
    let results = get_test_file_path("results.test");

    let run1 = Command::new(&bin)
        .args(["-m", "map", "-C", "--seed", "987654"])
        .arg(&qrels)
        .arg(&results)
        .output()
        .expect("Failed to run te-rust");

    let run2 = Command::new(&bin)
        .args(["-m", "map", "-C", "--seed", "987654"])
        .arg(&qrels)
        .arg(&results)
        .output()
        .expect("Failed to run te-rust");

    assert_eq!(String::from_utf8_lossy(&run1.stdout), String::from_utf8_lossy(&run2.stdout));
}
