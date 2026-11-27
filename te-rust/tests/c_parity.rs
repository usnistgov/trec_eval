use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const METADATA_SECTION: &str = "workspace.metadata.trec_eval_ref";

static C_TREC_EVAL_BIN: OnceLock<Result<PathBuf, String>> = OnceLock::new();

// Helper functions for finding things we need to run the C-parity tests: 
// - the workspace root,
// - the te-rust binary, and
// - the C trec_eval binary (either from TREC_EVAL_DIR or by building

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

fn get_te_rust_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_te-rust"))
}

fn get_test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test-data")
        .join(filename)
}

fn c_binary_name() -> String {
    format!("trec_eval{}", std::env::consts::EXE_SUFFIX)
}

fn is_valid_sha1(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

// Parse the pinned commit from the root Cargo.toml's [workspace.metadata.trec_eval_ref] section.
fn parse_pinned_commit(toml_content: &str) -> Result<String, String> {
    let doc: toml::Table = toml_content
        .parse()
        .map_err(|e| format!("Failed to parse root Cargo.toml: {}", e))?;

    let section = doc
        .get("workspace")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("trec_eval_ref"))
        .ok_or_else(|| format!("Missing [{}] section in root Cargo.toml", METADATA_SECTION))?;

    let commit_val = section.get("commit").ok_or_else(|| {
        format!(
            "Missing 'commit' key under [{}] in root Cargo.toml",
            METADATA_SECTION
        )
    })?;

    let commit = commit_val.as_str().ok_or_else(|| {
        format!(
            "Invalid [{}].commit in Cargo.toml: expected string, found {}",
            METADATA_SECTION,
            commit_val.type_str()
        )
    })?;

    if !is_valid_sha1(commit) {
        return Err(format!(
            "Invalid [{}].commit '{}': expected a full 40-character hexadecimal git SHA-1",
            METADATA_SECTION, commit
        ));
    }

    Ok(commit.to_string())
}

fn read_pinned_commit(ws_root: &Path) -> Result<String, String> {
    let manifest_path = ws_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "Failed to read root Cargo.toml at '{}': {}",
            manifest_path.display(),
            e
        )
    })?;
    parse_pinned_commit(&content)
}

// The user can override the C trec_eval binary location via TREC_EVAL_DIR. 
// If that is set, we check for the expected binary there.
fn resolve_override_dir(ws_root: &Path, dir_override: &str) -> Result<PathBuf, String> {
    let dir = ws_root.join(dir_override);
    let bin = dir.join(c_binary_name());
    if bin.is_file() {
        return Ok(bin);
    }
    Err(format!(
        "TREC_EVAL_DIR is set to '{}', but expected C trec_eval binary was not found at '{}'.\n\
         Build C trec_eval in '{}' (e.g., `make -C {}` on Unix or `build.bat` on Windows) or unset TREC_EVAL_DIR.",
        dir_override,
        bin.display(),
        dir.display(),
        dir.display()
    ))
}

// Make sure that the pinned commit is in the local git repository, and that we are running inside a git work tree.
fn verify_commit_in_git_repo(ws_root: &Path, commit: &str) -> Result<(), String> {
    let work_tree = Command::new("git")
        .arg("-C")
        .arg(ws_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|e| {
            format!(
                "Cannot run C-parity tests: failed to invoke `git` ({}).\n\
                 Install git or set TREC_EVAL_DIR=/path/to/built/trec_eval.",
                e
            )
        })?;

    if !work_tree.status.success() {
        return Err(format!(
            "Cannot run C-parity tests: '{}' is not inside a git repository.\n\
             Run from a git checkout containing commit {} or set TREC_EVAL_DIR=/path/to/built/trec_eval.",
            ws_root.display(),
            commit
        ));
    }

    let check_commit = Command::new("git")
        .arg("-C")
        .arg(ws_root)
        .args(["cat-file", "-e", &format!("{}^{{commit}}", commit)])
        .output()
        .map_err(|e| format!("Failed to invoke `git cat-file`: {}", e))?;

    if !check_commit.status.success() {
        return Err(format!(
            "Cannot run C-parity tests: pinned C trec_eval commit {commit} (configured in \
             Cargo.toml under [{section}]) is not present in this local git repository.\n\n\
             To fetch the required commit:\n  git fetch origin {commit}\n\
             (or in a shallow clone: git fetch --unshallow)\n\n\
             Alternatively, point TREC_EVAL_DIR at an existing built C trec_eval directory:\n  \
             TREC_EVAL_DIR=/path/to/trec_eval cargo test -p te-rust --test c_parity -- --ignored",
            commit = commit,
            section = METADATA_SECTION
        ));
    }

    Ok(())
}

struct PathCleanupGuard {
    path: Option<PathBuf>,
    is_dir: bool,
}

impl PathCleanupGuard {
    fn dir(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            is_dir: true,
        }
    }

    fn file(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            is_dir: false,
        }
    }

    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for PathCleanupGuard {
    fn drop(&mut self) {
        if let Some(p) = self.path.take() {
            if self.is_dir {
                let _ = std::fs::remove_dir_all(p);
            } else {
                let _ = std::fs::remove_file(p);
            }
        }
    }
}

// Now for the real fun: build the reference C trec_eval binary.
// In some ways this makes me want to ditch shell scripts forever in favor of Rust, but I digress.
fn materialize_and_build_c_trec_eval(
    ws_root: &Path,
    target_root: &Path,
    commit: &str,
) -> Result<PathBuf, String> {
    if cfg!(windows) {
        return Err(format!(
            "Cannot automatically build C trec_eval on native Windows: the Windows C build \
             (build.bat) requires MSVC and external Git submodules.\n\
             Either run the C-parity suite under WSL, or check out commit {commit} with \
             `--recurse-submodules`, build trec_eval.exe via `build.bat`, and run:\n  \
             TREC_EVAL_DIR=/path/to/trec_eval cargo test -p te-rust --test c_parity -- --ignored",
            commit = commit
        ));
    }

    let ref_root = target_root.join("trec_eval_ref");
    let cache_dir = ref_root.join(commit);
    let cached_bin = cache_dir.join(c_binary_name());
    if cached_bin.is_file() {
        return Ok(cached_bin);
    }

    verify_commit_in_git_repo(ws_root, commit)?;

    std::fs::create_dir_all(&ref_root).map_err(|e| {
        format!(
            "Failed to create cache directory '{}': {}",
            ref_root.display(),
            e
        )
    })?;

    let unique = format!("{}-{}", commit, std::process::id());
    let staging_dir = ref_root.join(format!(".tmp-{}", unique));
    let tar_path = ref_root.join(format!(".tmp-{}.tar", unique));

    let _ = std::fs::remove_dir_all(&staging_dir);
    let _ = std::fs::remove_file(&tar_path);

    std::fs::create_dir_all(&staging_dir).map_err(|e| {
        format!(
            "Failed to create staging directory '{}': {}",
            staging_dir.display(),
            e
        )
    })?;
    let mut staging_guard = PathCleanupGuard::dir(staging_dir.clone());
    let tar_guard = PathCleanupGuard::file(tar_path.clone());

    let archive_out = Command::new("git")
        .arg("-C")
        .arg(ws_root)
        .args(["archive", "--format=tar", "--output"])
        .arg(&tar_path)
        .arg(commit)
        .output()
        .map_err(|e| format!("Failed to invoke `git archive`: {}", e))?;
    if !archive_out.status.success() {
        return Err(format!(
            "`git archive` failed for commit {}: {}",
            commit,
            String::from_utf8_lossy(&archive_out.stderr)
        ));
    }

    let tar_out = Command::new("tar")
        .arg("-xf")
        .arg(&tar_path)
        .arg("-C")
        .arg(&staging_dir)
        .output()
        .map_err(|e| {
            format!(
                "Failed to invoke `tar` to extract C trec_eval archive: {}",
                e
            )
        })?;
    if !tar_out.status.success() {
        return Err(format!(
            "`tar` failed while extracting commit {}: {}",
            commit,
            String::from_utf8_lossy(&tar_out.stderr)
        ));
    }
    drop(tar_guard);

    let make_out = Command::new("make")
        .arg("-C")
        .arg(&staging_dir)
        .arg("trec_eval")
        .output()
        .map_err(|e| {
            format!(
                "Failed to invoke `make` to build C trec_eval ({}).\n\
                 C-parity tests require `make` and a C compiler, or a prebuilt TREC_EVAL_DIR.",
                e
            )
        })?;
    if !make_out.status.success() {
        return Err(format!(
            "`make -C {} trec_eval` failed (commit {}):\nstdout:\n{}\nstderr:\n{}",
            staging_dir.display(),
            commit,
            String::from_utf8_lossy(&make_out.stdout),
            String::from_utf8_lossy(&make_out.stderr)
        ));
    }

    let built_bin = staging_dir.join(c_binary_name());
    if !built_bin.is_file() {
        return Err(format!(
            "`make` succeeded but expected binary '{}' was not produced",
            built_bin.display()
        ));
    }

    match std::fs::rename(&staging_dir, &cache_dir) {
        Ok(()) => {
            staging_guard.disarm();
            Ok(cached_bin)
        }
        Err(rename_err) => {
            // Another process may have raced and populated cache_dir first.
            if cached_bin.is_file() {
                Ok(cached_bin)
            } else {
                Err(format!(
                    "Failed to move built C trec_eval from '{}' to '{}': {}",
                    staging_dir.display(),
                    cache_dir.display(),
                    rename_err
                ))
            }
        }
    }
}

fn resolve_c_trec_eval_bin() -> Result<PathBuf, String> {
    let ws_root = workspace_root();

    if let Ok(override_dir) = std::env::var("TREC_EVAL_DIR") {
        let trimmed = override_dir.trim();
        if !trimmed.is_empty() {
            return resolve_override_dir(&ws_root, trimmed);
        }
    }

    let commit = read_pinned_commit(&ws_root)?;
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| ws_root.join("target"));

    materialize_and_build_c_trec_eval(&ws_root, &target_root, &commit)
}

fn get_trec_eval_bin() -> PathBuf {
    C_TREC_EVAL_BIN
        .get_or_init(resolve_c_trec_eval_bin)
        .clone()
        .unwrap_or_else(|msg| panic!("{}", msg))
}

// Now, the actual tests!
// First, bits to compare trec_eval outputs

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

fn parse_ordered_rows(output_str: &str) -> Vec<(String, String, String)> {
    output_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                Some((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ))
            } else {
                None
            }
        })
        .collect()
}

fn compare_outputs(rust_args: &[&str], c_args: &[&str], qrels_name: &str, results_name: &str) {
    let qrels_path = get_test_data_path(qrels_name);
    let results_path = get_test_data_path(results_name);

    // 1. Run te-rust
    let rust_bin = get_te_rust_bin();
    let rust_output = Command::new(&rust_bin)
        .args(rust_args)
        .arg(&qrels_path)
        .arg(&results_path)
        .output()
        .expect("Failed to execute te-rust");
    assert!(
        rust_output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
    let rust_str = String::from_utf8_lossy(&rust_output.stdout);
    let rust_map = parse_output(&rust_str);

    // 2. Run C trec_eval
    let c_bin = get_trec_eval_bin();
    let c_output = Command::new(&c_bin)
        .args(c_args)
        .arg(&qrels_path)
        .arg(&results_path)
        .output()
        .expect("Failed to execute C trec_eval");
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

// --- Harness unit tests (run during plain `cargo test`, no C build required) ---

#[test]
fn test_workspace_cargo_toml_has_valid_trec_eval_ref() {
    let commit = read_pinned_commit(&workspace_root())
        .expect("root Cargo.toml must declare a valid [workspace.metadata.trec_eval_ref].commit");
    assert!(is_valid_sha1(&commit));
}

#[test]
fn test_parse_pinned_commit_handles_comments_and_other_sections() {
    let sample = r#"
[workspace]
members = ["te-rust"]

[workspace.metadata.trec_eval_ref] # reference commit
# inline note
commit = "f4253652c8efd0d86ddffd0d163cc0a0f813111a" # v10.0

[profile.release]
commit = "should-not-be-read"
"#;
    assert_eq!(
        parse_pinned_commit(sample).unwrap(),
        "f4253652c8efd0d86ddffd0d163cc0a0f813111a"
    );

    let inline_table = r#"
[workspace.metadata]
trec_eval_ref = { commit = "f4253652c8efd0d86ddffd0d163cc0a0f813111a" }
"#;
    assert_eq!(
        parse_pinned_commit(inline_table).unwrap(),
        "f4253652c8efd0d86ddffd0d163cc0a0f813111a"
    );
}

#[test]
fn test_parse_pinned_commit_rejects_invalid_inputs() {
    assert!(parse_pinned_commit("[workspace]\nmembers = []\n").is_err());
    assert!(parse_pinned_commit("[workspace.metadata.trec_eval_ref]\n").is_err());
    assert!(parse_pinned_commit("[workspace.metadata.trec_eval_ref]\ncommit = 12345\n").is_err());
    assert!(
        parse_pinned_commit("[workspace.metadata.trec_eval_ref]\ncommit = \"shortsha\"\n").is_err()
    );
    assert!(
        parse_pinned_commit(
            "[workspace.metadata.trec_eval_ref]\ncommit = \"zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\"\n"
        )
        .is_err()
    );
}

#[test]
fn test_missing_commit_error_guides_user() {
    let ws = workspace_root();
    let in_git = Command::new("git")
        .arg("-C")
        .arg(&ws)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_git {
        return;
    }

    let fake_sha = "0000000000000000000000000000000000000000";
    let err = verify_commit_in_git_repo(&ws, fake_sha)
        .expect_err("nonexistent commit should fail");
    assert!(err.contains(fake_sha), "error should cite SHA: {}", err);
    assert!(
        err.contains("git fetch origin"),
        "error should suggest git fetch: {}",
        err
    );
    assert!(
        err.contains("TREC_EVAL_DIR"),
        "error should mention TREC_EVAL_DIR: {}",
        err
    );
}

#[test]
fn test_non_git_directory_error_guides_user() {
    let dir = std::env::temp_dir().join(format!(
        "te_rust_non_git_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    let _guard = PathCleanupGuard::dir(dir.clone());

    let fake_sha = "f4253652c8efd0d86ddffd0d163cc0a0f813111a";
    let err = verify_commit_in_git_repo(&dir, fake_sha)
        .expect_err("non-git dir should fail");
    assert!(
        err.contains("TREC_EVAL_DIR"),
        "error should mention TREC_EVAL_DIR: {}",
        err
    );
}

#[test]
fn test_trec_eval_dir_override_missing_binary_error() {
    let err = resolve_override_dir(&workspace_root(), "nonexistent_trec_eval_dir_12345")
        .expect_err("missing override dir should fail");
    assert!(
        err.contains("TREC_EVAL_DIR"),
        "error should mention TREC_EVAL_DIR: {}",
        err
    );
    assert!(
        err.contains(&c_binary_name()),
        "error should mention expected binary name: {}",
        err
    );
}

// --- Live C-parity tests (opt-in via `-- --ignored`) ---

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_default() {
    compare_outputs(&[], &[], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_query_level() {
    compare_outputs(&["-q"], &["-q"], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_judged_only() {
    compare_outputs(&["-J"], &["-J"], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_max_docs() {
    compare_outputs(&["-M", "100"], &["-M", "100"], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_relevance_level() {
    compare_outputs(&["-l", "2"], &["-l", "2"], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_parametric_cutoffs() {
    compare_outputs(
        &["-m", "P.5", "-m", "ndcg_cut.10"],
        &["-m", "P.5", "-m", "ndcg_cut.10"],
        "qrels.test",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_stock_set() {
    compare_outputs(&["-m", "set"], &["-m", "set"], "qrels.test", "results.test");
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_specialized_standard_metrics() {
    compare_outputs(
        &["-m", "recall.5,10", "-m", "success.1,5", "-m", "11pt_avg", "-m", "utility", "-m", "ndcg"],
        &["-m", "recall.5,10", "-m", "success.1,5", "-m", "11pt_avg", "-m", "utility", "-m", "ndcg"],
        "qrels.test",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_relstring_query_level() {
    compare_outputs(
        &["-q", "-m", "relstring.10"],
        &["-q", "-m", "relstring.10"],
        "qrels.test",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_group_3() {
    compare_outputs(
        &["-m", "map_cut", "-m", "relative_P", "-m", "Rprec_mult", "-m", "iprec_at_recall"],
        &["-m", "map_cut", "-m", "relative_P", "-m", "Rprec_mult", "-m", "iprec_at_recall"],
        "qrels.test",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_group_4() {
    compare_outputs(
        &["-q", "-m", "gm_map", "-m", "gm_bpref", "-m", "infAP"],
        &["-q", "-m", "gm_map", "-m", "gm_bpref", "-m", "infAP"],
        "qrels.test",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_group_5_7() {
    // 1. Standard cases (includes experimental measures G, binG, yaap, which are
    //    excluded from groups but must still be verified against C trec_eval).
    compare_outputs(
        &[
            "-q",
            "-m", "unj",
            "-m", "num_nonrel_judged_ret",
            "-m", "rbp",
            "-m", "rbp_resid",
            "-m", "yaap",
            "-m", "G",
            "-m", "binG",
        ],
        &[
            "-q",
            "-m", "unj",
            "-m", "num_nonrel_judged_ret",
            "-m", "rbp",
            "-m", "rbp_resid",
            "-m", "yaap",
            "-m", "G",
            "-m", "binG",
        ],
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
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_qrels_jg() {
    compare_outputs(
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        "qrels.123",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_qrels_jg_with_comments() {
    compare_outputs(
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        &["-m", "qrels_jg", "-q", "-R", "qrels_jg"],
        "qrels.comments.123",
        "results.test",
    );
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_regression_unsorted_cutoffs() {
    let qrels_path = get_test_data_path("qrels.test");
    let results_path = get_test_data_path("results.test");
    let qrels_jg_path = get_test_data_path("qrels.123");

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

    let c_bin = get_trec_eval_bin();
    for (args, qrels) in cases {
        let rust_out = Command::new(get_te_rust_bin())
            .args(*args)
            .arg(qrels)
            .arg(&results_path)
            .output()
            .expect("Failed to run te-rust");
        assert!(
            rust_out.status.success(),
            "te-rust failed for {:?}: {}",
            args,
            String::from_utf8_lossy(&rust_out.stderr)
        );

        let c_out = Command::new(&c_bin)
            .args(*args)
            .arg(qrels)
            .arg(&results_path)
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
        assert_eq!(
            rust_rows.len(),
            c_rows.len(),
            "Row count mismatch for {:?}",
            args
        );
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
}

#[test]
#[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
fn test_c_parity_legal_parameter_edge_cases() {
    for spec in [
        "iprec_at_recall.1.",
        "iprec_at_recall.0",
        "set_F.0",
        "rbp.p=0",
        "rbp.p=0.5,1=2,2=5",
    ] {
        compare_outputs(&["-m", spec], &["-m", spec], "qrels.test", "results.test");
    }
    compare_outputs(
        &["-q", "-m", "relstring.0"],
        &["-q", "-m", "relstring.0"],
        "qrels.test",
        "results.test",
    );
}
