#![allow(dead_code)]
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyFile,
    InvalidFieldCount { line_num: usize, expected: usize, found: usize },
    InvalidInteger { line_num: usize, field: String, value: String },
    InvalidFloat { line_num: usize, field: String, value: String },
    DuplicateDocument { line_num: usize, qid: String, docno: String },
    RelevanceOutOfBounds { line_num: usize, value: i64 },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyFile => write!(f, "File is empty"),
            ParseError::InvalidFieldCount { line_num, expected, found } => {
                write!(f, "Line {}: expected at least {} fields, found {}", line_num, expected, found)
            }
            ParseError::InvalidInteger { line_num, field, value } => {
                write!(f, "Line {}: invalid integer for field '{}': '{}'", line_num, field, value)
            }
            ParseError::InvalidFloat { line_num, field, value } => {
                write!(f, "Line {}: invalid float for field '{}': '{}'", line_num, field, value)
            }
            ParseError::DuplicateDocument { line_num, qid, docno } => {
                write!(f, "Line {}: duplicate document '{}' for query '{}'", line_num, docno, qid)
            }
            ParseError::RelevanceOutOfBounds { line_num, value } => {
                write!(f, "Line {}: relevance score {} out of bounds (-1,000,000..1,000,000)", line_num, value)
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum IOError {
    FileReadError(io::Error),
    FormatError(ParseError),
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOError::FileReadError(e) => write!(f, "I/O Error: {}", e),
            IOError::FormatError(e) => write!(f, "Format Error: {}", e),
        }
    }
}

impl std::error::Error for IOError {}

/// Reads a grade as `trec_eval`'s `str_to_long` does (`get_qrels.c:18`): the leading integer, with
/// a warning when characters follow it, so `1.5` reads as 1 and `0.9` as 0. A value with no leading
/// integer, or one that does not fit in an `i64`, is an error.
fn parse_grade(value: &str, line_num: usize, warnings: &mut Vec<String>) -> Result<i64, IOError> {
    let invalid = || {
        IOError::FormatError(ParseError::InvalidInteger {
            line_num,
            field: "rel".to_string(),
            value: value.to_string(),
        })
    };
    let start = usize::from(value.starts_with(['+', '-']));
    let digits = value[start..].bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return Err(invalid());
    }
    let end = start + digits;
    let rel = value[..end].parse::<i64>().map_err(|_| invalid())?;
    if end < value.len() {
        warnings.push(format!("Line {}: value '{}' partially read as '{}'", line_num, value, rel));
    }
    Ok(rel)
}

/// Reads a score as `trec_eval`'s `strtod` does (`get_trec_results.c` with #125): the longest
/// leading number, with a warning when characters follow it, so `1.5x` reads as 1.5. Infinities
/// and hexadecimal values such as `0x1p3` are read as `strtod` reads them. A value outside the
/// range of an `f64`, such as `1e400` or `1e-400`, is kept as ±inf or ±0 with a warning. NaN, and
/// a value with no leading number, are errors.
fn parse_score(value: &str, line_num: usize, warnings: &mut Vec<String>) -> Result<f64, IOError> {
    let invalid = || {
        IOError::FormatError(ParseError::InvalidFloat {
            line_num,
            field: "sim".to_string(),
            value: value.to_string(),
        })
    };
    let (sim, len) = strtod_prefix(value).ok_or_else(invalid)?;
    if sim.is_nan() {
        return Err(invalid());
    }
    if out_of_range(&value[..len], sim) {
        warnings.push(format!("Line {}: score '{}' out of range, read as {}", line_num, value, sim));
    }
    if len < value.len() {
        warnings.push(format!("Line {}: score '{}' partially read as '{}'", line_num, value, &value[..len]));
    }
    Ok(sim)
}

/// Whether `strtod` sets `ERANGE` for the number `prefix` and returns its `value` of ±inf or ±0:
/// a finite number too large or too small for an `f64`. Some C libraries also set `ERANGE` for a
/// subnormal result, which is close to the number given; neither trec_eval nor te-rust warns then.
fn out_of_range(prefix: &str, value: f64) -> bool {
    let unsigned = prefix.trim_start_matches(['+', '-']);
    if value.is_infinite() {
        return !unsigned.starts_with(['i', 'I']);
    }
    if value != 0.0 {
        return false;
    }
    let (significand, exponent_marks) = match unsigned.strip_prefix("0x").or_else(|| unsigned.strip_prefix("0X")) {
        Some(hex) => (hex, ['p', 'P']),
        None => (unsigned, ['e', 'E']),
    };
    let significand = significand.split(exponent_marks).next().unwrap_or_default();
    significand.bytes().any(|b| b.is_ascii_hexdigit() && b != b'0')
}

/// The value and length of the longest prefix of `s` that C's `strtod` reads in the C locale, or
/// `None` when it reads nothing.
fn strtod_prefix(s: &str) -> Option<(f64, usize)> {
    let bytes = s.as_bytes();
    let sign = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let negative = bytes.first() == Some(&b'-');
    let rest = &bytes[sign..];
    let has_word = |word: &[u8]| rest.len() >= word.len() && rest[..word.len()].eq_ignore_ascii_case(word);
    let signed = |x: f64| if negative { -x } else { x };
    if has_word(b"infinity") {
        return Some((signed(f64::INFINITY), sign + 8));
    }
    if has_word(b"inf") {
        return Some((signed(f64::INFINITY), sign + 3));
    }
    if has_word(b"nan") {
        // strtod also reads an optional `(n-char-sequence)` after `nan`.
        let chars = rest[3..].iter().skip(1).take_while(|c| c.is_ascii_alphanumeric() || **c == b'_').count();
        let len = if rest.get(3) == Some(&b'(') && rest.get(4 + chars) == Some(&b')') { 5 + chars } else { 3 };
        return Some((f64::NAN, sign + len));
    }
    if rest.len() > 2 && rest[0] == b'0' && (rest[1] == b'x' || rest[1] == b'X') {
        if let Some((value, len)) = hex_prefix(&rest[2..]) {
            return Some((signed(value), sign + 2 + len));
        }
    }
    let int_digits = rest.iter().take_while(|c| c.is_ascii_digit()).count();
    let mut len = int_digits;
    if rest.get(len) == Some(&b'.') {
        let frac_digits = rest[len + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        if int_digits + frac_digits > 0 {
            len += 1 + frac_digits;
        }
    }
    if len == 0 {
        return None;
    }
    if let Some(b'e' | b'E') = rest.get(len) {
        let exp_sign = usize::from(matches!(rest.get(len + 1), Some(b'+' | b'-')));
        let exp_digits = rest[len + 1 + exp_sign..].iter().take_while(|c| c.is_ascii_digit()).count();
        if exp_digits > 0 {
            len += 1 + exp_sign + exp_digits;
        }
    }
    // Rust's parser rounds as strtod does, and gives an infinity or zero out of range.
    let value = s[..sign + len].parse::<f64>().ok()?;
    Some((value, sign + len))
}

/// The value and length of a hexadecimal significand and optional binary exponent, as `strtod`
/// reads them after `0x`, or `None` when no hexadecimal digit follows.
fn hex_prefix(rest: &[u8]) -> Option<(f64, usize)> {
    let mut mantissa: u64 = 0;
    let mut exponent: i64 = 0; // the value is mantissa * 2^exponent
    let mut sticky = false; // a nonzero digit did not fit in the mantissa
    let mut digits = 0;
    let mut point = false;
    let mut len = 0;
    while let Some(&c) = rest.get(len) {
        if c == b'.' && !point {
            point = true;
        } else if let Some(d) = char::from(c).to_digit(16) {
            digits += 1;
            if mantissa >> 60 == 0 {
                mantissa = mantissa << 4 | u64::from(d);
                exponent -= if point { 4 } else { 0 };
            } else {
                sticky |= d != 0;
                exponent += if point { 0 } else { 4 };
            }
        } else {
            break;
        }
        len += 1;
    }
    if digits == 0 {
        return None;
    }
    if let Some(b'p' | b'P') = rest.get(len) {
        let exp_sign = usize::from(matches!(rest.get(len + 1), Some(b'+' | b'-')));
        let start = len + 1 + exp_sign;
        let exp_digits = rest[start..].iter().take_while(|c| c.is_ascii_digit()).count();
        if exp_digits > 0 {
            let value = rest[start..start + exp_digits]
                .iter()
                .fold(0i64, |e, &c| (e * 10 + i64::from(c - b'0')).min(1 << 20));
            exponent += if rest[len + 1] == b'-' { -value } else { value };
            len = start + exp_digits;
        }
    }
    Some((round_binary(mantissa, sticky, exponent), len))
}

/// `mantissa * 2^exponent` rounded to the nearest `f64`, ties to even, where `sticky` means that
/// nonzero bits below the mantissa were dropped.
fn round_binary(mantissa: u64, sticky: bool, exponent: i64) -> f64 {
    if mantissa == 0 {
        return 0.0;
    }
    let zeros = mantissa.leading_zeros();
    let m = u128::from(mantissa << zeros);
    let top = exponent - i64::from(zeros) + 63; // the value lies in [2^top, 2^(top + 1))
    if top > 1023 {
        return f64::INFINITY;
    }
    let shift = (11 + (-1022 - top).max(0)).min(65) as u32;
    let (q, r, half) = (m >> shift, m & ((1u128 << shift) - 1), 1u128 << (shift - 1));
    let up = r > half || (r == half && (sticky || q & 1 == 1));
    let q = (q + u128::from(up)) as u64;
    if top < -1022 {
        return f64::from_bits(q); // subnormal; a carry into bit 52 gives the smallest normal
    }
    let (q, top) = if q >> 53 == 1 { (q >> 1, top + 1) } else { (q, top) };
    if top > 1023 {
        return f64::INFINITY;
    }
    f64::from_bits(((top + 1023) as u64) << 52 | (q & ((1u64 << 52) - 1)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrelsRecord {
    pub docno: String,
    pub rel: i64,
}

#[derive(Debug, Clone)]
pub struct QrelsQuery {
    pub qid: String,
    pub records: Vec<QrelsRecord>,
}

#[derive(Debug, Clone)]
pub struct QrelsData {
    pub queries: Vec<QrelsQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a grade read only in part.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrelsJGGroup {
    pub jg: String,
    pub records: Vec<QrelsRecord>,
}

#[derive(Debug, Clone)]
pub struct QrelsJGQuery {
    pub qid: String,
    pub groups: Vec<QrelsJGGroup>,
}

#[derive(Debug, Clone)]
pub struct QrelsJGData {
    pub queries: Vec<QrelsJGQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a grade read only in part.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunRecord {

    pub docno: String,
    pub sim: f64,
}

#[derive(Debug, Clone)]
pub struct RunQuery {
    pub qid: String,
    pub records: Vec<RunRecord>,
}

#[derive(Debug, Clone)]
pub struct RunData {
    pub run_id: String,
    pub queries: Vec<RunQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a score read only in part.
    pub warnings: Vec<String>,
}

struct RawQrels {
    qid: String,
    docno: String,
    rel: i64,
    line_num: usize,
}

struct RawRun {
    qid: String,
    docno: String,
    sim: f64,
    run_id: String,
    line_num: usize,
}

/// True for the field separators of trec_eval's parsers: C `isspace` in the C locale.
/// `char::is_whitespace` also matches non-ASCII characters such as U+00A0, which
/// can occur inside a docno.
fn is_field_sep(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

fn split_fields(line: &str) -> Vec<&str> {
    line.split(is_field_sep).filter(|field| !field.is_empty()).collect()
}

/// Parses standard qrels relevance judgments file format.
pub fn parse_trec_qrels<P: AsRef<Path>>(path: P) -> Result<QrelsData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    let reader = BufReader::new(file);
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut line_num = 0;
    let mut warnings = Vec::new();

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim_matches(is_field_sep);

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts = split_fields(trimmed);
        if parts.len() != 4 {
            return Err(IOError::FormatError(ParseError::InvalidFieldCount {
                line_num,
                expected: 4,
                found: parts.len(),
            }));
        }

        let qid = parts[0].to_string();
        let docno = parts[2].to_string();
        let rel_str = parts[3];

        let rel = parse_grade(rel_str, line_num, &mut warnings)?;

        if rel < -1_000_000 || rel > 1_000_000 {
            return Err(IOError::FormatError(ParseError::RelevanceOutOfBounds {
                line_num,
                value: rel,
            }));
        }

        raw_records.push(RawQrels {
            qid,
            docno,
            rel,
            line_num,
        });
    }

    if raw_records.is_empty() {
        return Err(IOError::FormatError(ParseError::EmptyFile));
    }

    // Sort by qid first, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
            other => mt_cmp_order(other),
        }
    });

    // Validate duplicate documents under same query
    for i in 1..raw_records.len() {
        if raw_records[i].qid == raw_records[i - 1].qid && raw_records[i].docno == raw_records[i - 1].docno {
            return Err(IOError::FormatError(ParseError::DuplicateDocument {
                line_num: raw_records[i].line_num,
                qid: raw_records[i].qid.clone(),
                docno: raw_records[i].docno.clone(),
            }));
        }
    }

    // Group into QrelsQuery records in a linear pass
    let mut queries = Vec::new();
    let mut current_query = QrelsQuery {
        qid: raw_records[0].qid.clone(),
        records: vec![QrelsRecord {
            docno: raw_records[0].docno.clone(),
            rel: raw_records[0].rel,
        }],
    };

    for raw in raw_records.into_iter().skip(1) {
        if raw.qid == current_query.qid {
            current_query.records.push(QrelsRecord {
                docno: raw.docno,
                rel: raw.rel,
            });
        } else {
            queries.push(current_query);
            current_query = QrelsQuery {
                qid: raw.qid,
                records: vec![QrelsRecord {
                    docno: raw.docno,
                    rel: raw.rel,
                }],
            };
        }
    }
    queries.push(current_query);

    Ok(QrelsData { queries, comments, warnings })
}

struct RawQrelsJG {
    qid: String,
    jg: String,
    docno: String,
    rel: i64,
    line_num: usize,
}

/// Parses judgment groups qrels format (qid  jg  docno  rel).
pub fn parse_trec_qrels_jg<P: AsRef<Path>>(path: P) -> Result<QrelsJGData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    let reader = BufReader::new(file);
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut line_num = 0;
    let mut warnings = Vec::new();

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim_matches(is_field_sep);

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts = split_fields(trimmed);
        if parts.len() != 4 {
            return Err(IOError::FormatError(ParseError::InvalidFieldCount {
                line_num,
                expected: 4,
                found: parts.len(),
            }));
        }

        let qid = parts[0].to_string();
        let jg = parts[1].to_string();
        let docno = parts[2].to_string();
        let rel_str = parts[3];

        let rel = parse_grade(rel_str, line_num, &mut warnings)?;

        if rel < -1_000_000 || rel > 1_000_000 {
            return Err(IOError::FormatError(ParseError::RelevanceOutOfBounds {
                line_num,
                value: rel,
            }));
        }

        raw_records.push(RawQrelsJG {
            qid,
            jg,
            docno,
            rel,
            line_num,
        });
    }

    if raw_records.is_empty() {
        return Err(IOError::FormatError(ParseError::EmptyFile));
    }

    // Sort by qid first, then jg, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => match a.jg.cmp(&b.jg) {
                std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
                other => mt_cmp_order(other),
            },
            other => mt_cmp_order(other),
        }
    });

    // Validate duplicate documents under same query and judgment group
    for i in 1..raw_records.len() {
        if raw_records[i].qid == raw_records[i - 1].qid
            && raw_records[i].jg == raw_records[i - 1].jg
            && raw_records[i].docno == raw_records[i - 1].docno
        {
            return Err(IOError::FormatError(ParseError::DuplicateDocument {
                line_num: raw_records[i].line_num,
                qid: raw_records[i].qid.clone(),
                docno: raw_records[i].docno.clone(),
            }));
        }
    }

    // Group into QrelsJGQuery and QrelsJGGroup in a linear pass
    let mut queries: Vec<QrelsJGQuery> = Vec::new();

    for raw in raw_records {
        if let Some(last_q) = queries.last_mut().filter(|q| q.qid == raw.qid) {
            if let Some(last_g) = last_q.groups.last_mut().filter(|g| g.jg == raw.jg) {
                last_g.records.push(QrelsRecord {
                    docno: raw.docno,
                    rel: raw.rel,
                });
            } else {
                last_q.groups.push(QrelsJGGroup {
                    jg: raw.jg,
                    records: vec![QrelsRecord {
                        docno: raw.docno,
                        rel: raw.rel,
                    }],
                });
            }
        } else {
            queries.push(QrelsJGQuery {
                qid: raw.qid,
                groups: vec![QrelsJGGroup {
                    jg: raw.jg,
                    records: vec![QrelsRecord {
                        docno: raw.docno,
                        rel: raw.rel,
                    }],
                }],
            });
        }
    }

    Ok(QrelsJGData { queries, comments, warnings })
}

/// Parses standard results (run) file format.

pub fn parse_trec_run<P: AsRef<Path>>(path: P) -> Result<RunData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    let reader = BufReader::new(file);
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut warnings = Vec::new();
    let mut line_num = 0;
    let mut last_run_id = String::new();

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim_matches(is_field_sep);

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts = split_fields(trimmed);
        if parts.len() < 6 {
            return Err(IOError::FormatError(ParseError::InvalidFieldCount {
                line_num,
                expected: 6,
                found: parts.len(),
            }));
        }

        let qid = parts[0].to_string();
        let docno = parts[2].to_string();
        let sim_str = parts[4];
        let run_id = parts[5].to_string();

        let sim = parse_score(sim_str, line_num, &mut warnings)?;

        last_run_id = run_id.clone();

        raw_records.push(RawRun {
            qid,
            docno,
            sim,
            run_id,
            line_num,
        });
    }

    if raw_records.is_empty() {
        return Err(IOError::FormatError(ParseError::EmptyFile));
    }

    // Sort raw records by qid first, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
            other => mt_cmp_order(other),
        }
    });

    // Validate duplicate documents under same query
    for i in 1..raw_records.len() {
        if raw_records[i].qid == raw_records[i - 1].qid && raw_records[i].docno == raw_records[i - 1].docno {
            return Err(IOError::FormatError(ParseError::DuplicateDocument {
                line_num: raw_records[i].line_num,
                qid: raw_records[i].qid.clone(),
                docno: raw_records[i].docno.clone(),
            }));
        }
    }

    // Group into RunQuery records in a linear pass
    let mut queries = Vec::new();
    let mut current_query = RunQuery {
        qid: raw_records[0].qid.clone(),
        records: vec![RunRecord {
            docno: raw_records[0].docno.clone(),
            sim: raw_records[0].sim,
        }],
    };

    for raw in raw_records.into_iter().skip(1) {
        if raw.qid == current_query.qid {
            current_query.records.push(RunRecord {
                docno: raw.docno,
                sim: raw.sim,
            });
        } else {
            queries.push(current_query);
            current_query = RunQuery {
                qid: raw.qid,
                records: vec![RunRecord {
                    docno: raw.docno,
                    sim: raw.sim,
                }],
            };
        }
    }
    queries.push(current_query);

    Ok(RunData {
        run_id: last_run_id,
        queries,
        comments,
        warnings,
    })
}

#[inline]
fn mt_cmp_order(ord: std::cmp::Ordering) -> std::cmp::Ordering {
    ord
}

#[cfg(test)]
mod tests {
    use super::{out_of_range, strtod_prefix};

    /// Each case's value and length are those of the C library's `strtod` (macOS 26, arm64).
    #[test]
    fn strtod_prefix_reads_what_strtod_reads() {
        let cases: &[(&str, f64, usize)] = &[
            ("1.5", 1.5, 3),
            ("1.5x", 1.5, 3),
            ("5e", 5.0, 1),
            ("5e+", 5.0, 1),
            ("+.5e1", 5.0, 5),
            (".5", 0.5, 2),
            ("5.", 5.0, 2),
            ("-0", -0.0, 2),
            ("1e400", f64::INFINITY, 5),
            ("-1e400", f64::NEG_INFINITY, 6),
            ("1e-400", 0.0, 6),
            ("inf", f64::INFINITY, 3),
            ("-Infinity", f64::NEG_INFINITY, 9),
            ("infinit", f64::INFINITY, 3),
            ("0x10", 16.0, 4),
            ("0x1p3", 8.0, 5),
            ("-0X1P-2", -0.25, 7),
            ("0x1.8p1", 3.0, 7),
            ("0x.8", 0.5, 4),
            ("0x", 0.0, 1),
            ("0xg", 0.0, 1),
            ("0x1p", 1.0, 3),
            ("0x1p+", 1.0, 3),
            ("0x1.0000000000001p0", 1.0000000000000002, 19),
            // Halfway cases round to even, and a nonzero digit past the mantissa breaks the tie.
            ("0x1.00000000000008p0", 1.0, 20),
            ("0x1.00000000000018p0", 1.0000000000000004, 20),
            ("0x1.000000000000080000000001p0", 1.0000000000000002, 30),
            ("0x1.fffffffffffffp1023", f64::MAX, 22),
            ("0x1.fffffffffffff8p1023", f64::INFINITY, 23),
            ("0x1p-1074", 5e-324, 9),
            ("0x1.8p-1074", 1e-323, 11),
            ("0x1p-1075", 0.0, 9),
        ];
        for &(s, value, len) in cases {
            let (v, n) = strtod_prefix(s).unwrap_or_else(|| panic!("{s} not read"));
            assert_eq!((v.to_bits(), n), (value.to_bits(), len), "{s}: {v}");
        }
    }

    /// Whether the C library's `strtod` sets `ERANGE` and returns ±inf or ±0 (macOS 26, arm64).
    #[test]
    fn out_of_range_matches_strtod_erange() {
        let cases = [
            ("1e400", true),
            ("-1e400", true),
            ("1.7976931348623159e308", true),
            ("1.7976931348623157e308", false),
            ("1e-400", true),
            ("-1e-400", true),
            ("1e-324", true),
            ("3e-324", false),
            ("1e-310", false),
            ("0e-400", false),
            ("0x1p2000", true),
            ("0x1p-1075", true),
            ("0x1p-1074", false),
            ("0x0p-5000", false),
            ("inf", false),
            ("-Infinity", false),
        ];
        for (s, expected) in cases {
            let (value, len) = strtod_prefix(s).unwrap();
            assert_eq!(out_of_range(&s[..len], value), expected, "{s}");
        }
    }

    #[test]
    fn strtod_prefix_reads_nan_and_nothing_as_strtod_does() {
        for (s, len) in [("nan", 3), ("-NaN", 4), ("nan(12ab)", 9), ("nan(", 3), ("nan()", 5)] {
            let (v, n) = strtod_prefix(s).unwrap();
            assert!(v.is_nan() && n == len, "{s}: {v} {n}");
        }
        for s in ["", ".", "+", "-", "e5", "in", "abc", "x1"] {
            assert_eq!(strtod_prefix(s), None, "{s}");
        }
    }
}
