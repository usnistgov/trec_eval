use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rbp-relevance-level")
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

#[test]
fn binary_topic_does_not_move_with_level() {
    // gain(a) = 1 at rank 1, so rbp = (1 - 0.9) * 1.
    for level in ["1", "2", "3", "4", "100"] {
        let out = te_rust(&[
            "-l",
            level,
            "-m",
            "rbp",
            &fixture("binary.qrels"),
            &fixture("binary.run"),
        ]);
        assert_eq!(summary(&out, "rbp"), "0.1000", "-l {level}");
    }
}

#[test]
fn graded_topic_does_not_move_with_level() {
    // Gains normalise by the top grade 3: d = 1, a = 2/3, c = 1/3, at ranks 1, 3 and 5.
    // 0.1 * (1 + 2/3 * 0.81 + 1/3 * 0.6561) = 0.1759
    for level in ["1", "2", "3", "4", "5", "100"] {
        let out = te_rust(&[
            "-l",
            level,
            "-m",
            "rbp",
            &fixture("graded.qrels"),
            &fixture("graded.run"),
        ]);
        assert_eq!(summary(&out, "rbp"), "0.1759", "-l {level}");
    }
}
