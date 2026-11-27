# C `trec_eval` Reference Pinning & Parity Test Harness Design

## 1. Problem Statement

On the `trec_eval_v11` branch, live C-parity regression tests in `te-rust/tests/regression.rs` currently rely on two different copies of the C `trec_eval` codebase:

1. **Locally:** 123 C source and test files (plus three Windows submodules in `.gitmodules`) are committed directly into the `trec_eval/` directory on this branch, even though `.gitignore` ignores `/trec_eval/`.
2. **In CI:** `.github/workflows/test-rust.yml` ignores `trec_eval/` and checks out a hardcoded commit (`f4253652c8efd0d86ddffd0d163cc0a0f813111a`, `v10.0`) from `usnistgov/trec_eval` into `trec_eval_ref`, builds it with `make`, and passes `TREC_EVAL_DIR=trec_eval_ref`.

This causes several problems:
- **Drift and duplication:** Committing a snapshot of the C source into `trec_eval_v11` duplicates history that already lives in the same repository (`main` / release commits) and creates a second reference that drifts from CI and from upstream C fixes.
- **Heavy default `cargo test` requirements:** Plain `cargo test` currently requires a pre-built C `trec_eval` binary, a C compiler, and `make`, even though live C-parity verification is primarily a core-developer workflow rather than a requirement for normal users or downstream packagers.
- **Mixed test concerns:** `te-rust/tests/regression.rs` mixes live C-comparison tests with pure-Rust CLI regression tests (help flags, exit codes, issue `#107` float validation, issue `#102`/`#97`/`#98`/`#142`/`#143`/`#94` relevance-level handling), and reads test inputs (`qrels.test`, `results.test`, `qrels.123`, `qrels.comments.123`) from `TREC_EVAL_DIR/test/` despite identical copies already living in `te-rust/tests/test-data/`.

This became a major concern because a number of issues were discovered by Jeff Dalton using Claude, both on the C side and the Rust side, and keeping the regression tests in sync while different PRs were in flight on each side was going to be impossible in the existing setup.

---

## 2. Goals and Non-Goals

### Goals
1. Remove the committed `trec_eval/` directory and its C-only Windows submodule entries from `.gitmodules` on `trec_eval_v11`.
2. Establish a **single source of truth** for the pinned C `trec_eval` reference commit in root `Cargo.toml` (`[workspace.metadata.trec_eval_ref]`), starting at `f4253652c8efd0d86ddffd0d163cc0a0f813111a` (`v10.0`) so this refactor stays isolated from any behavior changes on `main`.
3. Make plain `cargo test` run **pure Rust tests only** (unit tests, confidence intervals, and pure-Rust CLI regression tests), with zero dependency on `git`, `make`, a C compiler, or network access.
4. Provide an explicit opt-in invocation for core developers and CI to run live C-parity tests via Rust's `#[ignore]` mechanism (`cargo test -p te-rust --test c_parity -- --ignored`).
5. Automatically materialize (via `git archive`) and build the pinned C commit into Cargo's `target/` directory, caching by commit SHA.
6. When the pinned commit is not present in the local git repository, fail clearly with a diagnostic telling the user which commit is missing and the exact commands to fix it (no silent network fetches during `cargo test`).
7. Drive `.github/workflows/test-rust.yml` from the same `Cargo.toml` pin so CI exercises the exact same harness path as local developers.

### Non-Goals / Follow-Up Work
- **Bumping the pinned C reference commit to current `main`:** Deferred to a separate follow-up PR once this harness is in place.
- **Native Windows C auto-build inside `c_parity.rs`:** C `trec_eval`'s Windows build (`build.bat`) requires MSVC (`cl.exe`) plus three external Git submodules (`mman-win32`, `ya_getopt`, `strings_h`) that `git archive` does not populate. Plain `cargo test` works natively on Windows, and Windows users/CI run `c_parity` via WSL or by building `trec_eval.exe` and setting `TREC_EVAL_DIR`.

---

## 3. Configuration: Single Source of Truth in Root `Cargo.toml`

Cargo explicitly reserves `[workspace.metadata.*]` for custom workspace tooling and ignores it without warnings. Placing the pin in the workspace root `Cargo.toml` (rather than `te-rust/Cargo.toml`) keeps developer/CI test metadata out of the `te-rust` package manifest and makes it equally accessible to workspace tests and GitHub Actions.

```toml
[workspace.metadata.trec_eval_ref]
# Pinned C trec_eval commit used by live C-parity tests (te-rust/tests/c_parity.rs).
# Currently pinned to v10.0. Update this SHA (and note the release/PR here) to
# advance the C parity baseline.
commit = "f4253652c8efd0d86ddffd0d163cc0a0f813111a"
```

### Updating the Reference Commit
To update the C reference in the future:
1. Edit `commit` (and the adjacent comment) in root `Cargo.toml`.
2. Run `cargo test -p te-rust --test c_parity -- --ignored`.
   - Because the build cache is keyed by full commit SHA (`target/trec_eval_ref/<commit>/`), bumping `commit` automatically triggers a clean extraction and rebuild without stale artifacts.
3. Because `.github/workflows/test-rust.yml` already triggers on `Cargo.toml` changes and reads `commit` from it, CI picks up the new reference automatically with no workflow edits.

### Parsing `Cargo.toml`
`te-rust/Cargo.toml` adds `toml = "1"` under `[dev-dependencies]` (MSRV 1.85, matching CI, with zero impact on the release binary, library consumers, or `te-python` wheels). `te-rust/tests/c_parity.rs` parses root `Cargo.toml` via `toml::Table` and:
- Looks up `workspace.metadata.trec_eval_ref.commit` (handling standard tables, dotted keys, inline tables, and comments).
- Validates that `commit` is a 40-character hexadecimal SHA-1 string (rejecting short SHAs, branch names, or path-unsafe strings).
- Is covered by regular (non-ignored) unit tests inside `c_parity.rs`, including a test that parses the real workspace `Cargo.toml`, so plain `cargo test` immediately catches a malformed pin.

---

## 4. Test Suite Restructuring

### 4.1 Test Fixtures (`te-rust/tests/test-data/`)
All four files currently loaded via `get_test_file_path()` from `TREC_EVAL_DIR/test/` (`qrels.test`, `results.test`, `qrels.123`, `qrels.comments.123`) are already byte-for-byte identical in `te-rust/tests/test-data/`. Both `cli_regression.rs` and `c_parity.rs` will read test inputs exclusively from `te-rust/tests/test-data/` (relative to `CARGO_MANIFEST_DIR`). The checked-out C tree is used *only* to build and run the reference `trec_eval` executable.

### 4.2 Splitting `te-rust/tests/regression.rs`
Replace `te-rust/tests/regression.rs` with two focused integration test targets:

1. **`te-rust/tests/cli_regression.rs`** — pure Rust CLI regression tests, run by plain `cargo test`:
   - `test_regression_help_flags`
   - `test_regression_parameter_validation_and_exit_codes` (exit-1 and exit-2 validation loops, plus exit-0 checks for legal edge cases)
   - `test_regression_11pt_avg_sorted_recall_levels` (split from `test_regression_unsorted_cutoffs`; tests intentional divergence from C `#142`)
   - `test_regression_run_score_validation_issue_107`
   - `test_regression_rel_levels_and_issue_102`

2. **`te-rust/tests/c_parity.rs`** — live C-parity comparison tests, each annotated with:
   ```rust
   #[test]
   #[ignore = "requires C trec_eval reference; run with: cargo test -p te-rust --test c_parity -- --ignored"]
   ```
   Tests moved here:
   - `test_regression_default`
   - `test_regression_query_level`
   - `test_regression_judged_only`
   - `test_regression_max_docs`
   - `test_regression_relevance_level`
   - `test_regression_parametric_cutoffs`
   - `test_regression_stock_set`
   - `test_regression_specialized_standard_metrics`
   - `test_regression_relstring_query_level`
   - `test_regression_group_3`
   - `test_regression_group_4`
   - `test_regression_group_5_7`
   - `test_regression_qrels_jg`
   - `test_regression_qrels_jg_with_comments`
   - `test_regression_unsorted_cutoffs` (the 11 C-comparison cutoff cases)
   - `test_c_parity_legal_parameter_edge_cases` (the 6 legal-parameter C-comparison cases extracted from `test_regression_parameter_validation_and_exit_codes`)
   - Plus non-ignored unit tests for the `Cargo.toml` metadata parser and SHA validator.

### Why `#[ignore]` Over a Cargo Feature
- Does not add a test-only feature to `te-rust`'s public Cargo feature surface.
- Will not accidentally activate when downstream tools run `cargo test --all-features`.
- Still type-checks `c_parity.rs` during plain `cargo test`, preventing silent compile bit-rot.

---

## 5. C Reference Resolution, Extraction, and Caching

`c_parity.rs` resolves the C `trec_eval` binary once per test process via:

```rust
static C_TREC_EVAL_BIN: OnceLock<Result<PathBuf, String>> = OnceLock::new();
```

Storing `Result<PathBuf, String>` (rather than panicking inside `get_or_init`) ensures initialization runs strictly once even on failure: if the commit is missing or `make` fails, parallel test threads do not re-run `git`/`make` repeatedly and all report the same clear diagnostic.

### Resolution Order

1. **`TREC_EVAL_DIR` environment variable (explicit override):**
   - If set and non-empty, resolve `<workspace_root>.join(TREC_EVAL_DIR)` (absolute paths are preserved by `Path::join`), and look for `format!("trec_eval{}", std::env::consts::EXE_SUFFIX)` (`trec_eval` on Unix, `trec_eval.exe` on Windows).
   - If the binary does not exist, return an error naming the expected path and instructing the user to build C `trec_eval` in `<dir>` (or unset `TREC_EVAL_DIR`).
   - Useful for testing against an uncommitted local C working tree, a custom C build, or a Windows CI job that built `trec_eval.exe` via `build.bat`.

2. **Native Windows guard (when `TREC_EVAL_DIR` is unset):**
   - On `#[cfg(windows)]`, fail immediately with an explicit message explaining that automatic extraction/compilation is Unix-only (C `trec_eval` on Windows requires MSVC and Git submodules via `build.bat`), and instruct the user either to run under WSL or to clone the pinned `<commit>` with `--recurse-submodules`, build `trec_eval.exe` via `build.bat`, and set `TREC_EVAL_DIR`.

3. **Cached build under Cargo's target directory:**
   - Read `commit` from `<workspace_root>/Cargo.toml`.
   - Target root: `CARGO_TARGET_DIR` if set, else `<workspace_root>/target`.
   - Cache directory: `<target_root>/trec_eval_ref/<commit>/`.
   - Binary path: `<cache_dir>/trec_eval`.
   - If the cached binary already exists as a file, return it immediately (zero `git` or `make` cost on repeat runs).

4. **Verify local git repository and commit availability (no automatic network fetch):**
   - Run `git -C <workspace_root> rev-parse --is-inside-work-tree`.
     - If `git` cannot be executed or the workspace is not a git repository (e.g., extracted release tarball), return an error explaining that C-parity tests require a git checkout of `usnistgov/trec_eval` or `TREC_EVAL_DIR=/path/to/built/trec_eval`.
   - Run `git -C <workspace_root> cat-file -e <commit>^{commit}`.
     - If non-zero, return a clear actionable warning/error:
       ```text
       Cannot run C-parity tests: pinned C trec_eval commit <commit> (configured in
       Cargo.toml under [workspace.metadata.trec_eval_ref]) is not present in this
       local git repository.

       To fetch the required commit:
         git fetch origin <commit>
       (or in a shallow clone: git fetch --unshallow)

       Alternatively, point TREC_EVAL_DIR at an existing built C trec_eval directory:
         TREC_EVAL_DIR=/path/to/trec_eval cargo test -p te-rust --test c_parity -- --ignored
       ```

5. **Materialize and build in an atomic staging directory:**
   - Create a process-unique staging directory: `<target_root>/trec_eval_ref/.tmp-<commit>-<pid>/` (protected by a RAII drop guard that removes it on any failure).
   - Extract the commit without touching the working tree or `.git/worktrees`:
     - `git -C <workspace_root> archive --format=tar --output=<tar_path> <commit>`
     - `tar -xf <tar_path> -C <staging_dir>`
     - Remove `<tar_path>`.
     - Using `git archive` (instead of `git worktree`) leaves no git metadata behind when `cargo clean` deletes `target/`.
   - Build C `trec_eval`:
     - `make -C <staging_dir> trec_eval`
     - On failure (or missing `make`), include `make`'s stderr in the error and note that C-parity tests require `make` and a C compiler (or a prebuilt `TREC_EVAL_DIR`).
   - Atomically promote `<staging_dir>` → `<cache_dir>` via `std::fs::rename`:
     - If rename fails because another concurrent process populated `<cache_dir>` first and `<cache_dir>/trec_eval` exists, discard `<staging_dir>` and use `<cache_dir>/trec_eval`.
     - Otherwise surface the rename error.

---

## 6. Repository Cleanup & CI Integration

### 6.1 Git Tree Cleanup
- Remove all tracked files and submodule gitlinks under `trec_eval/` (`git rm -r trec_eval`).
- Remove `.gitmodules` (all three entries — `mman-win32`, `ya_getopt`, `strings_h` — are exclusively for building C `trec_eval` on Windows).
- Keep `/trec_eval/` in `.gitignore` so developers who manually place a local C checkout there won't dirty `git status`.

### 6.2 GitHub Actions (`.github/workflows/test-rust.yml`)
Run across `os: [ubuntu-latest, macos-latest, windows-latest]` and `rust-version: ["1.85.0", "stable"]`, reading `commit` from root `Cargo.toml` via `cargo metadata` + `jq` (avoiding any dependency on Python 3.11+ `tomllib` across OS images):

```yaml
      - name: Read pinned C trec_eval commit
        id: trec_eval_ref
        shell: bash
        run: |
          COMMIT=$(cargo metadata --no-deps --format-version 1 | jq -er '.metadata.trec_eval_ref.commit' | tr -d '\r')
          echo "commit=$COMMIT" >> "$GITHUB_OUTPUT"

      - name: Run Rust tests
        run: cargo test

      - name: Fetch pinned C trec_eval commit (Unix)
        if: runner.os != 'Windows'
        run: git fetch --no-tags --depth=1 origin "${{ steps.trec_eval_ref.outputs.commit }}"

      - name: Run C-parity regression tests (Unix)
        if: runner.os != 'Windows'
        run: cargo test -p te-rust --test c_parity -- --ignored

      - name: Check out C trec_eval reference (Windows)
        if: runner.os == 'Windows'
        uses: actions/checkout@v4
        with:
          repository: usnistgov/trec_eval
          ref: ${{ steps.trec_eval_ref.outputs.commit }}
          path: trec_eval_ref
          submodules: recursive

      - name: Set up MSVC (Windows)
        if: runner.os == 'Windows'
        uses: ilammy/msvc-dev-cmd@v1

      - name: Build C trec_eval (Windows)
        if: runner.os == 'Windows'
        shell: cmd
        working-directory: trec_eval_ref
        run: build.bat

      - name: Run C-parity regression tests (Windows)
        if: runner.os == 'Windows'
        env:
          TREC_EVAL_DIR: trec_eval_ref
        run: cargo test -p te-rust --test c_parity -- --ignored
```

This achieves three things simultaneously:
1. `cargo test` (which uses `default-members = ["te-rust"]`, leaving `te-python` to `test-python.yml` where Python is configured) runs *before* any C commit fetch/checkout, proving plain `cargo test` passes purely in Rust across Linux, macOS, and Windows.
2. On Linux and macOS, `cargo test -p te-rust --test c_parity -- --ignored` runs without `TREC_EVAL_DIR`, exercising the exact `Cargo.toml` → `git archive` → `make` → cache path on every PR.
3. On Windows, CI checks out the exact same `commit` with `submodules: recursive`, builds `trec_eval.exe` via `ilammy/msvc-dev-cmd@v1` + `build.bat`, and runs `c_parity` via `TREC_EVAL_DIR=trec_eval_ref`, exercising the override path and native Windows C-parity.

### 6.3 Documentation Updates
- `README.md`: remove `trec_eval/` from the Project Structure tree; document plain `cargo test` vs `cargo test -p te-rust --test c_parity -- --ignored`, `TREC_EVAL_DIR`, and how to bump `[workspace.metadata.trec_eval_ref]`.
- `AGENTS.md`: note that the C `trec_eval` source is no longer checked into `trec_eval/` on this branch and is referenced via root `Cargo.toml`.

---

## 7. Verification Plan

1. **Plain Rust tests:** `cargo test` passes with no `trec_eval/` directory present and reports the `c_parity` tests as ignored (and passes the non-ignored metadata parser unit tests).
2. **C-parity cold build:** `cargo test -p te-rust --test c_parity -- --ignored` extracts `f4253652c8efd0d86ddffd0d163cc0a0f813111a` via `git archive`, compiles into `target/trec_eval_ref/f4253652c8efd0d86ddffd0d163cc0a0f813111a/trec_eval`, and passes all 16 C-parity tests.
3. **C-parity warm cache:** Re-running `cargo test -p te-rust --test c_parity -- --ignored` reuses the cached binary without re-extracting or rebuilding.
4. **Missing commit error path:** Tested via unit test (or temporary fake SHA) confirming a clear diagnostic message with `git fetch origin <commit>` and `TREC_EVAL_DIR` guidance is emitted once.
5. **`TREC_EVAL_DIR` override:** Running `TREC_EVAL_DIR=<cached_dir> cargo test -p te-rust --test c_parity -- --ignored` uses the override binary, and pointing `TREC_EVAL_DIR` at a nonexistent path fails with a clear message.
6. **Python suite:** `uv run --project te-python --extra dev pytest te-python/tests` continues to pass unaffected.
