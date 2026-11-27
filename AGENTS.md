Guidance for AI coding agents, on initiating the trec_eval-in-Rust project.

## Project overview

The goal of this project is to design a clean-room reimplementation of trec_eval in Rust. Moving to a modern language will make maintenance easier and afford new capabilities, like exposing a Python API.

- **trec_eval (C reference)**: The C `trec_eval` source is no longer checked into a `trec_eval/` directory on this branch. Live C-parity tests (`te-rust/tests/c_parity.rs`) automatically extract and build the pinned commit configured under `[workspace.metadata.trec_eval_ref]` in the root `Cargo.toml` into `target/trec_eval_ref/<commit>/` when invoked with `cargo test -p te-rust --test c_parity -- --ignored` (or use `TREC_EVAL_DIR` to override).
- **te-rust**: This is the directory for the new trec_eval-in-Rust project.

The new tool will be compatible with existing trec_eval usage:
- The same command-line arguments, with the same semantics
- (There may be new arguments, but they can't conflict with existing ones)
- Accept files in the same formats
- The same measures

Scores obtained from te-rust should be identical to results from trec_eval. When differences exist they must be attributable to numerical differences.

## Workflow

Each stage begins with a design discussion, and creates a design document before any code may be written.

Stages and features are coded in branches, and folded back into the main branch once they have been tested.

## Testing

### Running Tests

- **Basic Rust tests (unit, confidence intervals, pure-Rust CLI regression, and C-parity harness checks):**
  ```bash
  cargo test
  ```
  `Cargo.toml` sets `default-members = ["te-rust"]`, so plain `cargo test` runs all pure-Rust tests without requiring C `trec_eval` or Python. Avoid `cargo test --workspace` unless Python's `libpython` is on the linker/loader path (on macOS, `te-python` requires `DYLD_LIBRARY_PATH` and `PYO3_PYTHON`).

- **C-parity regression tests (live comparison against pinned C `trec_eval`):**
  ```bash
  cargo test -p te-rust --test c_parity -- --ignored
  ```
  Automatically extracts (via `git archive`) and builds the commit pinned under `[workspace.metadata.trec_eval_ref]` in the root `Cargo.toml` into `target/trec_eval_ref/<commit>/`.
  - To test against an existing local C `trec_eval` build (also required on native Windows):
    ```bash
    TREC_EVAL_DIR=/path/to/trec_eval cargo test -p te-rust --test c_parity -- --ignored
    ```

- **Python tests (`te-python`):**
  ```bash
  uv run --project te-python --extra dev pytest te-python/tests
  ```
  After modifying Rust code used by `te-python`, rebuild the extension before running `pytest`:
  ```bash
  uv run --project te-python --extra dev maturin develop --manifest-path te-python/Cargo.toml
  ```
  To run `te-python`'s Rust unit tests directly (`cargo test -p te-python --lib`), ensure `PYO3_PYTHON` points to the Python executable (and on macOS, `DYLD_LIBRARY_PATH` includes that Python's `LIBDIR`).

### Test Requirements

All components will have unit tests. For measures, these tests must check for corner cases in the measure calculation, such as having no relevant documents for a topic. For files, these tests should check for formatting errors.

For metrics, there should be a common set of test cases that incorporate common edge cases:
- no documents retrieved (an empty ranking)
- no relevant documents retrieved
- no judged documents retrieved (i.e., no documents in the ranking are present in the qrels file)
- A single relevant or nonrelevant document retrieved.
- Cases designed to elicit numerical errors like rounding, infinity, or not-a-number results.

Then individual metrics may add cases that are specific to that metric.

## AI Agent Behavior

You are most helpful when you propose options, describe pros and cons, challenge my preconceived notions, and make constructive suggestions. It is not helpful to tell me that I am brilliant, or otherwise stroke my ego, or describe features or code as beautiful. I am not interested in your opinion. I am interested in your ideas.
