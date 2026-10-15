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

/// Parses standard qrels relevance judgments file format.
pub fn parse_trec_qrels<P: AsRef<Path>>(path: P) -> Result<QrelsData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    let reader = BufReader::new(file);
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut line_num = 0;

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
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

        let rel = rel_str.parse::<i64>().map_err(|_| {
            IOError::FormatError(ParseError::InvalidInteger {
                line_num,
                field: "rel".to_string(),
                value: rel_str.to_string(),
            })
        })?;

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

    Ok(QrelsData { queries, comments })
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

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
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

        let rel = rel_str.parse::<i64>().map_err(|_| {
            IOError::FormatError(ParseError::InvalidInteger {
                line_num,
                field: "rel".to_string(),
                value: rel_str.to_string(),
            })
        })?;

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

    Ok(QrelsJGData { queries, comments })
}

/// Parses standard results (run) file format.

pub fn parse_trec_run<P: AsRef<Path>>(path: P) -> Result<RunData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    let reader = BufReader::new(file);
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut line_num = 0;
    let mut last_run_id = String::new();

    for line_res in reader.lines() {
        line_num += 1;
        let line = line_res.map_err(IOError::FileReadError)?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
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

        let sim = sim_str.parse::<f64>().map_err(|_| {
            IOError::FormatError(ParseError::InvalidFloat {
                line_num,
                field: "sim".to_string(),
                value: sim_str.to_string(),
            })
        })?;

        if !sim.is_finite() {
            return Err(IOError::FormatError(ParseError::InvalidFloat {
                line_num,
                field: "sim".to_string(),
                value: sim_str.to_string(),
            }));
        }

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
    })
}

#[inline]
fn mt_cmp_order(ord: std::cmp::Ordering) -> std::cmp::Ordering {
    ord
}
