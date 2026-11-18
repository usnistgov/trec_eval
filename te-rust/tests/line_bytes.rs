//! Lines with bytes that would change their fields: CR line endings, NUL bytes and bytes that
//! are not UTF-8. The files with those bytes are written at test time, so that the committed
//! fixtures stay text.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/line-bytes")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// Writes `bytes` to a file in Cargo's temporary directory for integration tests.
fn written(name: &str, bytes: &[u8]) -> String {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("line-bytes-{name}"));
    std::fs::write(&path, bytes).expect("failed to write a test file");
    path.to_string_lossy().into_owned()
}

/// `abc.run` and `abc.qrels` with each LF replaced by `ending`.
fn with_endings(name: &str, ending: &str) -> String {
    let text = std::fs::read_to_string(fixture(name)).unwrap();
    written(
        &format!("{ending:?}-{name}"),
        text.replace('\n', ending).as_bytes(),
    )
}

fn te_rust(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .output()
        .expect("failed to run te-rust")
}

fn summary(output: &Output, measure: &str) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 3 && fields[0] == measure && fields[1] == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))[2]
        .to_string()
}

fn refusal(args: &[&str]) -> String {
    let out = te_rust(args);
    assert!(
        !out.status.success(),
        "te-rust accepted {args:?}: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_run_with_cr_line_endings_is_refused() {
    // Read as one line, the run held only document a (num_ret 1, map 0.5000), and both
    // te-rust and trec_eval exited 0.
    let run = with_endings("abc.run", "\r");
    let stderr = refusal(&["-m", "num_ret", &fixture("abc.qrels"), &run]);
    assert!(
        stderr.contains("Line 1: CR character without LF"),
        "{stderr}"
    );
}

#[test]
fn qrels_with_cr_line_endings_get_the_same_message() {
    // Before, the message counted the fields of the joined line: "found 12".
    let qrels = with_endings("abc.qrels", "\r");
    let stderr = refusal(&["-m", "map", &qrels, &fixture("abc.run")]);
    assert!(
        stderr.contains("Line 1: CR character without LF"),
        "{stderr}"
    );
    let stderr = refusal(&[
        "-R",
        "qrels_jg",
        "-m",
        "map_avgjg",
        &qrels,
        &fixture("abc.run"),
    ]);
    assert!(
        stderr.contains("Line 1: CR character without LF"),
        "{stderr}"
    );
}

#[test]
fn a_cr_before_other_whitespace_is_refused() {
    // Only a CR directly before the LF ends a line.
    let run = written("cr-space.run", b"1 Q0 a 1 3 r\r \n1 Q0 b 2 2 r\n");
    let stderr = refusal(&["-m", "map", &fixture("abc.qrels"), &run]);
    assert!(
        stderr.contains("Line 1: CR character without LF"),
        "{stderr}"
    );
}

#[test]
fn crlf_files_are_read_as_before() {
    // a and c are relevant at ranks 1 and 3: AP = (1/1 + 2/3) / 2 = 0.8333.
    let qrels = with_endings("abc.qrels", "\r\n");
    let run = with_endings("abc.run", "\r\n");
    let out = te_rust(&["-m", "num_ret", "-m", "map", &qrels, &run]);
    assert_eq!(summary(&out, "num_ret"), "3");
    assert_eq!(summary(&out, "map"), "0.8333");
    // A CR that ends the file is accepted, as trec_eval accepts it.
    let run = written(
        "trailing-cr.run",
        b"1 Q0 a 1 3 r\n1 Q0 b 2 2 r\n1 Q0 c 3 1 r\r",
    );
    let out = te_rust(&["-m", "map", &qrels, &run]);
    assert_eq!(summary(&out, "map"), "0.8333");
}

#[test]
fn a_nul_byte_is_refused_naming_the_line() {
    // Before, the NUL was part of the field: the docno d1<NUL>x counted as another document,
    // with exit 0. trec_eval with #126 refuses these files with exit 2.
    let qrels = written("nul-field.qrels", b"1 0 a 1\n1 0 b\0x 0\n");
    let stderr = refusal(&["-m", "num_rel", &qrels, &fixture("abc.run")]);
    assert!(stderr.contains("Line 2: NUL character"), "{stderr}");

    let run = written("nul-field.run", b"1 Q0 a\0x 1 3 r\n");
    let stderr = refusal(&["-m", "num_ret", &fixture("abc.qrels"), &run]);
    assert!(stderr.contains("Line 1: NUL character"), "{stderr}");

    let run = written("nul-line.run", b"1 Q0 a 1 3 r\n\0\n1 Q0 c 3 1 r\n");
    let stderr = refusal(&["-m", "num_ret", &fixture("abc.qrels"), &run]);
    assert!(stderr.contains("Line 2: NUL character"), "{stderr}");
}

#[test]
fn a_data_line_that_is_not_utf8_is_refused_naming_the_line() {
    // Before: "stream did not contain valid UTF-8", with no line. The Latin-1 byte 0xE9 is
    // byte 7 of line 2.
    let run = written("latin1.run", b"1 Q0 a 1 3 r\n1 Q0 d\xe9 2 2 r\n");
    let stderr = refusal(&["-m", "map", &fixture("abc.qrels"), &run]);
    assert!(
        stderr.contains("Line 2: invalid UTF-8 at byte 7"),
        "{stderr}"
    );

    let qrels = written("latin1.qrels", b"1 0 d\xe9 1\n");
    let stderr = refusal(&["-m", "map", &qrels, &fixture("abc.run")]);
    assert!(
        stderr.contains("Line 1: invalid UTF-8 at byte 6"),
        "{stderr}"
    );
}

#[test]
fn a_comment_line_may_hold_bytes_that_are_not_utf8() {
    // The comment is ignored, so the scores are those of abc.qrels: AP = 0.8333.
    let qrels = written(
        "latin1-comment.qrels",
        b"# judged by Ren\xe9\n1 0 a 1\n1 0 b 0\n1 0 c 1\n",
    );
    let out = te_rust(&["-m", "map", &qrels, &fixture("abc.run")]);
    assert_eq!(summary(&out, "map"), "0.8333");
}

#[test]
fn nul_and_lone_cr_are_refused_in_a_comment_line() {
    // Before, both comments were skipped and the run scored AP = 0.8333, with exit 0.
    let run = written("comment-nul.run", b"# header \0 x\n1 Q0 a 1 3 r\n");
    let stderr = refusal(&["-m", "map", &fixture("abc.qrels"), &run]);
    assert!(stderr.contains("Line 1: NUL character"), "{stderr}");

    let run = written("comment-cr.run", b"# header\r# more\n1 Q0 a 1 3 r\n");
    let stderr = refusal(&["-m", "map", &fixture("abc.qrels"), &run]);
    assert!(
        stderr.contains("Line 1: CR character without LF"),
        "{stderr}"
    );
}
