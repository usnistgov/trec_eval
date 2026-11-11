#![cfg(unix)]

use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn test_data(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/test-data")
        .join(name)
}

/// A pipe whose reader has already exited, so the first write must fail.
fn pipe_with_no_reader() -> Stdio {
    let mut reader = Command::new("true")
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to run true");
    let writer = reader.stdin.take().unwrap();
    reader.wait().unwrap();
    Stdio::from(writer)
}

#[test]
fn closed_stdout_ends_with_sigpipe_and_no_panic() {
    let output = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(["-q", "-m", "all_trec"])
        .arg(test_data("qrels.test"))
        .arg(test_data("results.test"))
        .stdout(pipe_with_no_reader())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert_eq!(output.status.signal(), Some(13), "{:?}", output.status);
}
