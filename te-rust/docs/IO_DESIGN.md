# `te-rust` I/O and Parsing Design Document

This document outlines the detailed design of the Input/Output (I/O) and parsing routines for the `te-rust` project, a Rust reimplementation of `trec_eval`. The primary goal is to parse standard TREC format files strictly and accurately, matching the behaviors, validation rules, and tie-breaking ordering of the original C codebase.

---

## 1. File Formats and Strictness Specifications

### 1.1 Qrels (Standard Relevance Judgments) File Format
*   **Format**: Space or tab-separated lines containing exactly 4 fields:
    ```text
    qid  iter  docno  rel
    ```
    *   `qid`: Query/topic ID (`String`).
    *   `iter`: Query iteration/context (`String`, ignored).
    *   `docno`: Document ID (`String`).
    *   `rel`: Relevance judgment (`i64` between -1,000,000 and 1,000,000).
*   **Comments & Blank Lines**: Skipped. Comments start with `#` in the first column.
*   **Strictness**: Must have exactly 4 fields. Extra non-whitespace trailing characters are rejected as malformed.

### 1.2 QrelsJG (Judgment Group Relevance) File Format
*   **Format**: Space or tab-separated lines containing exactly 4 fields:
    ```text
    qid  ujg  docno  rel
    ```
    *   `qid`: Query/topic ID (`String`).
    *   `ujg`: User judgment group (`String`).
    *   `docno`: Document ID (`String`).
    *   `rel`: Relevance judgment (`i64` between -1,000,000 and 1,000,000).
*   **Comments & Blank Lines**: Skipped. Comments start with `#` in the first column.
*   **Strictness**: Must have exactly 4 fields. Extra non-whitespace trailing characters are rejected as malformed.

### 1.3 Prefs (User Preferences) File Format
*   **Format**: Space or tab-separated lines containing exactly 5 fields:
    ```text
    qid  ujg  ujsubg  docno  rel_level
    ```
    *   `qid`: Query/topic ID (`String`).
    *   `ujg`: User judgment group (`String`).
    *   `ujsubg`: User judgment sub-group (`String`).
    *   `docno`: Document ID (`String`).
    *   `rel_level`: Relevance level (`f64`).
*   **Comments & Blank Lines**: Skipped. Comments start with `#` in the first column.
*   **Strictness**: Must have exactly 5 fields. Extra non-whitespace trailing characters are rejected as malformed. `rel_level` must be a valid finite float (no `NaN` or `Infinity`).

### 1.4 QrelsPrefs (Preference-style Qrels) File Format
*   **Format**: Space or tab-separated lines containing exactly 4 fields:
    ```text
    qid  ujg  docno  rel_level
    ```
    *   `qid`: Query/topic ID (`String`).
    *   `ujg`: User judgment group (`String`).
    *   `docno`: Document ID (`String`).
    *   `rel_level`: Relevance level (`f64`).
*   **Comments & Blank Lines**: Skipped. Comments start with `#` in the first column.
*   **Strictness**: Must have exactly 4 fields. Extra non-whitespace trailing characters are rejected as malformed. `rel_level` must be a valid finite float (no `NaN` or `Infinity`).

### 1.5 Results (Run) File Format
*   **Format**: Space or tab-separated lines containing at least 6 fields:
    ```text
    qid  iter  docno  rank  sim  run_id  [optional_fields...]
    ```
    *   `qid`: Query/topic ID (`String`).
    *   `iter`: Iteration/context (`String`, ignored).
    *   `docno`: Document ID (`String`).
    *   `rank`: Rank assigned by the system (`i64`, ignored).
    *   `sim`: Similarity score (`f64`).
    *   `run_id`: Run identifier (`String`).
*   **Comments & Blank Lines**: Skipped. Comments start with `#` in the first column.
*   **Strictness**: Must have at least 6 fields. Extra fields beyond `run_id` are allowed and ignored. `sim` must be a valid finite float (no `NaN` or `Infinity`).
*   **Run ID**: The global run ID is assigned from the `run_id` of the **last** parsed content line of the file.

---

## 2. Internal Data Models

To model the parsed data cleanly and prepare it for calculation, we store records in hierarchical flat arrays grouped by Query ID and other relevant subgroups.

```rust
// ==================== 1. Standard Qrels Format ====================

/// Represents a single relevance judgment record for a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrelsRecord {
    pub docno: String,
    pub rel: i64,
}

/// Relevance judgments grouped under a single Query ID.
#[derive(Debug, Clone)]
pub struct QrelsQuery {
    pub qid: String,
    pub records: Vec<QrelsRecord>,
}

/// Structured relevance judgments containing a list of query groups.
#[derive(Debug, Clone)]
pub struct QrelsData {
    pub queries: Vec<QrelsQuery>,
    pub comments: Vec<String>, // Stores parsed file comment lines (stripped of leading '#')
}

// ==================== 2. QrelsJG Format ====================

/// Relevance judgments grouped under a User Judgment Group (ujg).
#[derive(Debug, Clone)]
pub struct QrelsJGGroup {
    pub jg: String,
    pub records: Vec<QrelsRecord>,
}

/// QrelsJG judgments grouped under a single Query ID.
#[derive(Debug, Clone)]
pub struct QrelsJGQuery {
    pub qid: String,
    pub jgs: Vec<QrelsJGGroup>,
}

/// Structured QrelsJG data.
#[derive(Debug, Clone)]
pub struct QrelsJGData {
    pub queries: Vec<QrelsJGQuery>,
    pub comments: Vec<String>, // Stores parsed file comment lines
}

// ==================== 3. Prefs Format ====================

/// A single preference record containing a document and its relevance level.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefsRecord {
    pub docno: String,
    pub rel_level: f64,
}

/// Preferences grouped under a User Judgment Sub-Group (jsg).
#[derive(Debug, Clone)]
pub struct PrefsSubGroup {
    pub jsg: String,
    pub records: Vec<PrefsRecord>,
}

/// Preferences grouped under a User Judgment Group (jg).
#[derive(Debug, Clone)]
pub struct PrefsGroup {
    pub jg: String,
    pub sub_groups: Vec<PrefsSubGroup>,
}

/// Prefs data grouped under a single Query ID.
#[derive(Debug, Clone)]
pub struct PrefsQuery {
    pub qid: String,
    pub groups: Vec<PrefsGroup>,
}

/// Structured Prefs data.
#[derive(Debug, Clone)]
pub struct PrefsData {
    pub queries: Vec<PrefsQuery>,
    pub comments: Vec<String>, // Stores parsed file comment lines
}

// ==================== 4. QrelsPrefs Format ====================

/// Preference relevance level grouped under a User Judgment Group (jg).
#[derive(Debug, Clone)]
pub struct QrelsPrefsGroup {
    pub jg: String,
    pub records: Vec<PrefsRecord>,
}

/// QrelsPrefs grouped under a single Query ID.
#[derive(Debug, Clone)]
pub struct QrelsPrefsQuery {
    pub qid: String,
    pub groups: Vec<QrelsPrefsGroup>,
}

/// Structured QrelsPrefs data.
#[derive(Debug, Clone)]
pub struct QrelsPrefsData {
    pub queries: Vec<QrelsPrefsQuery>,
    pub comments: Vec<String>, // Stores parsed file comment lines
}

// ==================== 5. Run Results Format ====================

/// Represents a single run result record for a document.
#[derive(Debug, Clone, PartialEq)]
pub struct RunRecord {
    pub docno: String,
    pub sim: f64,
}

/// Run results grouped under a single Query ID.
#[derive(Debug, Clone)]
pub struct RunQuery {
    pub qid: String,
    pub records: Vec<RunRecord>,
}

/// Structured run results containing a list of query groups.
#[derive(Debug, Clone)]
pub struct RunData {
    pub run_id: String,
    pub queries: Vec<RunQuery>,
    pub comments: Vec<String>, // Stores parsed file comment lines
}
```


---

## 3. Tie-Breaking and Sorting Logic

In information retrieval, different ranking scores can result in identical similarity values (`sim`). `trec_eval` breaks similarity ties deterministically using document IDs (`docno`).

### 3.1 C `trec_eval` Sort Order (`comp_sim_docno`):
1.  **Similarity (`sim`)**: Primary sort key, descending (higher similarity scores first).
2.  **Document ID (`docno`)**: Secondary sort key, **descending lexicographical order** (alphabetically greater document IDs first).

> [!IMPORTANT]
> The secondary sort by `docno` is descending lexicographical order (i.e. `p2->docno` compared against `p1->docno` in C's `strcmp`). To match `trec_eval`'s exact evaluations, `te-rust` must implement the exact same tie-breaking logic during metrics computation or post-parsing ranking.

> [!WARNING]
> **Rust Floating-Point Sorting (`f64`)**:
> In Rust, standard floating-point numbers (`f64`) do not implement `Ord` (only `PartialOrd`) because `NaN` values cannot be ordered. Since `NaN` values are invalid as similarity scores in standard evaluations, we must handle this safely. Standard methods like `.sort_by()` with `partial_cmp().unwrap()` are safe *only* if the parser strictly validates that similarity scores are valid finite numbers (i.e. rejecting `NaN` and potentially `Infinity` values during parsing).

---

## 4. Parser Architecture and Error Handling

 We implement a hand-written parser using standard library features. It avoids third-party dependency overhead while providing precise syntax control.

### 4.1 Error Typing
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyFile,
    InvalidFieldCount { line_num: usize, expected: usize, found: usize },
    InvalidInteger { line_num: usize, field: String, value: String },
    InvalidFloat { line_num: usize, field: String, value: String },
    DuplicateDocument { line_num: usize, qid: String, docno: String },
}

#[derive(Debug)]
pub enum IOError {
    FileReadError(std::io::Error),
    FormatError(ParseError),
}
```

### 4.2 Parsing Steps (Example for Qrels)
1. Read the file line-by-line (using `BufReader` and `.lines()`).
2. Track the 1-based line number.
3. Trim leading and trailing whitespace.
4. If a line begins with `#` (after optional whitespace trimming), strip the leading `#` character and store the remainder of the line in the `comments` vector of our top-level data structure. Then skip to the next line.
5. If the line is empty, skip it.
6. Tokenize the line by splitting on whitespace.
7. Validate field count:
    * For Qrels: Must be exactly 4 tokens.
    * For Run/Results: Must be at least 6 tokens.
8. Parse integers and floats safely using standard `.parse::<i64>()` and `.parse::<f64>()`. For relevance values (`rel`), validate that they fall within a generous and safe range of `-1,000,000` to `1,000,000` to support modern preference-based dense scales while permanently preventing excessive vector allocations.
9. Store records sequentially.
10. Populate the `QrelsData` or `RunData` structures, sorting internal lists if necessary or leaving sorting as an explicit post-processing step before metric calculations. Keep all comments preserved in the `comments` field.

---

## 5. Testing Plan

We will implement a thorough suite of unit and integration tests to enforce formatting correctness and edge case handling.

### 5.1 Format Test Cases
*   **Valid Qrels**: Standard space/tab formatting, trailing spaces/tabs on lines.
*   **Valid Runs**: Extra trailing fields, whitespace variations, and missing scores resolved safely.
*   **Malformed Qrels**:
    *   Fewer than 4 fields.
    *   More than 4 fields (non-whitespace characters after `rel`).
    *   Non-integer relevance scores (e.g. `2.5`, `abc`).
*   **Malformed Runs**:
    *   Fewer than 6 fields.
    *   Non-float similarity score (e.g. `high`, `NaN`).
*   **Comments & Empty Lines**: Mixed comments, blank lines, trailing/leading blank lines, CRLF vs LF line endings.

### 5.2 Edge Cases
*   Empty files.
*   Zero queries or zero documents.
*   Single-entry files.
