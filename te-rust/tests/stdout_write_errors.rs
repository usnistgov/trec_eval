#![cfg(unix)]

use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

// trec_eval's status when it cannot print a measure. trec_eval itself ignores
// write errors and exits 0.
const EXIT_OUTPUT: i32 = 8;

fn test_data(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/test-data")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// Run te-rust with stdout redirected to `stdout`.
fn te_rust(args: &[&str], stdout: Stdio) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .stdout(stdout)
        .output()
        .expect("failed to run te-rust")
}

/// Run te-rust with stdout on a regular file that may not grow: with SIGXFSZ
/// ignored, every write fails with EFBIG, as writes fail on a full disk.
fn te_rust_with_file_size_limit(args: &[&str], name: &str) -> Output {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let stdout = File::create(&path).expect("failed to create the output file");
    Command::new("sh")
        .arg("-c")
        .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" \"$@\"")
        .arg(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .stdout(stdout)
        .output()
        .expect("failed to run sh")
}

fn assert_write_failure(output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(
        stderr.contains("te-rust: cannot write to standard output"),
        "{stderr}"
    );
    assert_eq!(
        output.status.code(),
        Some(EXIT_OUTPUT),
        "{:?}",
        output.status
    );
}

#[test]
fn failed_result_write_exits_with_a_message() {
    let output = te_rust_with_file_size_limit(
        &["-q", &test_data("qrels.test"), &test_data("results.test")],
        "results.out",
    );
    assert_write_failure(&output);
}

#[test]
fn failed_help_write_exits_with_a_message() {
    assert_write_failure(&te_rust_with_file_size_limit(
        &["--help-measures"],
        "help-measures.out",
    ));
    assert_write_failure(&te_rust_with_file_size_limit(
        &["--help-measure", "map"],
        "help-measure.out",
    ));
}

#[test]
fn failed_message_write_still_exits_8() {
    // With stderr on the same full disk, the message is lost but the status is not.
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("both.out");
    let stdout = File::create(&path).expect("failed to create the output file");
    let output = Command::new("sh")
        .arg("-c")
        .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" \"$@\" 2>&1")
        .arg(env!("CARGO_BIN_EXE_te-rust"))
        .args([&test_data("qrels.test"), &test_data("results.test")])
        .stdout(stdout)
        .output()
        .expect("failed to run sh");
    assert_eq!(
        output.status.code(),
        Some(EXIT_OUTPUT),
        "{:?}",
        output.status
    );
}

/// The exit status of te-rust with stderr on `stderr`, where stdout is a pipe.
fn status_with_stderr(args: &[&str], stderr: Stdio) -> Option<i32> {
    Command::new("sh")
        .arg("-c")
        .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" \"$@\"")
        .arg(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .stderr(stderr)
        .output()
        .expect("failed to run sh")
        .status
        .code()
}

#[test]
fn failed_error_message_write_keeps_the_status() {
    let qrels = test_data("qrels.test");
    for (args, name) in [
        (&[qrels.as_str(), "absent.run"][..], "absent-run.err"),
        (&["--help-measure", "nosuch"][..], "unknown-measure.err"),
    ] {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
        let full = File::create(&path).expect("failed to create the error file");
        let expected = status_with_stderr(args, Stdio::piped());
        assert_ne!(expected, Some(0), "{args:?}");
        assert_eq!(
            status_with_stderr(args, Stdio::from(full)),
            expected,
            "{args:?}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn full_device_exits_with_a_message() {
    let full = File::options()
        .write(true)
        .open("/dev/full")
        .expect("failed to open /dev/full");
    let output = te_rust(
        &[&test_data("qrels.test"), &test_data("results.test")],
        Stdio::from(full),
    );
    assert_write_failure(&output);
}

#[test]
fn successful_writes_are_unchanged() {
    let output = te_rust(
        &[
            "-m",
            "num_q",
            &test_data("qrels.test"),
            &test_data("results.test"),
        ],
        Stdio::piped(),
    );
    assert!(output.status.success(), "{:?}", output.status);
    // qrels.test and results.test share topics 301, 302 and 303.
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "num_q                 \tall\t3\n"
    );
    assert!(output.stderr.is_empty());
}
