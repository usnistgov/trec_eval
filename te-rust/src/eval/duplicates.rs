//! Repeated documents, checked where `trec_eval` checks them.
//!
//! `trec_eval` looks for a repeated document only when a measure forms a topic's ranking
//! (`te_form_res_rels`, `form_res_rels.c:128-150`): among the run documents that remain after
//! the cut to `-M`, and among the topic's judgments. A run topic that the qrels lack, a repeat
//! beyond `-M`, and a request for measures that form no ranking, such as `num_q`, are never
//! checked. The parsers therefore keep repeated documents, and the CLI and the Python
//! `Evaluator` call these functions before they evaluate.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::io::{QrelsData, QrelsJGData, QrelsRecord, RunData, RunQuery, RunRecord};
use crate::metrics::EvalConfig;

/// A document that appears twice in a topic that would be scored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoredDuplicate {
    Qrels { qid: String, docno: String },
    Run { qid: String, docno: String },
}

impl std::fmt::Display for ScoredDuplicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (ScoredDuplicate::Qrels { qid, docno } | ScoredDuplicate::Run { qid, docno }) = self;
        write!(f, "duplicate document '{}' for query '{}'", docno, qid)
    }
}

impl std::error::Error for ScoredDuplicate {}

/// The first repeated document in a topic that `evaluate_run` scores: the qrels topics that the
/// run has, or every qrels topic with `-c`. Topics are taken in qrels order, and in each the
/// run is checked before the qrels, as `trec_eval` does.
pub fn find_scored_duplicate(
    qrels: &QrelsData,
    run: &RunData,
    config: &EvalConfig,
) -> Option<ScoredDuplicate> {
    let run_by_qid = run_topics(run);
    qrels.queries.iter().find_map(|qrels_q| {
        let run_q = run_by_qid.get(qrels_q.qid.as_str());
        if run_q.is_none() && !config.average_complete_flag {
            return None;
        }
        run_q
            .and_then(|run_q| repeated_run_docno(run_q, config.max_num_docs_per_topic))
            .map(|docno| ScoredDuplicate::Run {
                qid: qrels_q.qid.clone(),
                docno,
            })
            .or_else(|| {
                repeated_qrels_docno(&qrels_q.records).map(|docno| ScoredDuplicate::Qrels {
                    qid: qrels_q.qid.clone(),
                    docno,
                })
            })
    })
}

/// The first repeated run document in a topic that `evaluate_run_jg` scores. The qrels_jg parser
/// refuses repeated judgments when it reads the file, as `trec_eval` does.
pub fn find_scored_duplicate_jg(
    qrels: &QrelsJGData,
    run: &RunData,
    config: &EvalConfig,
) -> Option<ScoredDuplicate> {
    let run_by_qid = run_topics(run);
    qrels.queries.iter().find_map(|qrels_q| {
        let run_q = run_by_qid.get(qrels_q.qid.as_str())?;
        repeated_run_docno(run_q, config.max_num_docs_per_topic).map(|docno| ScoredDuplicate::Run {
            qid: qrels_q.qid.clone(),
            docno,
        })
    })
}

fn run_topics(run: &RunData) -> HashMap<&str, &RunQuery> {
    run.queries.iter().map(|q| (q.qid.as_str(), q)).collect()
}

/// A docno that appears twice among the first `max_docs` documents of the ranking, which orders
/// by score and then by docno, both descending, as `align_query` does.
fn repeated_run_docno(run_q: &RunQuery, max_docs: usize) -> Option<String> {
    let mut ranked: Vec<&RunRecord> = run_q.records.iter().collect();
    if ranked.len() > max_docs {
        ranked.sort_by(|a, b| match b.sim.partial_cmp(&a.sim) {
            Some(Ordering::Equal) => b.docno.cmp(&a.docno),
            Some(other) => other,
            None => Ordering::Equal,
        });
        ranked.truncate(max_docs);
    }
    first_repeat(ranked.iter().map(|r| r.docno.as_str()))
}

fn repeated_qrels_docno(records: &[QrelsRecord]) -> Option<String> {
    first_repeat(records.iter().map(|r| r.docno.as_str()))
}

fn first_repeat<'a>(mut docnos: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut seen = HashSet::new();
    docnos
        .find(|docno| !seen.insert(*docno))
        .map(str::to_string)
}
