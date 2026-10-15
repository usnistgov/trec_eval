use crate::io::{QrelsJGQuery, QrelsQuery, RunQuery};

pub const RELVALUE_NONPOOL: i64 = -1;

pub const RELVALUE_UNJUDGED: i64 = -2;

/// Aligned evaluation state for a single query topic.
#[derive(Debug, Clone)]
pub struct QueryEvalState {
    /// The unique query topic ID.
    pub qid: String,
    /// The global run label.
    pub run_id: String,

    /// Aligned document list containing relevance judgments at each retrieved rank.
    /// Index matches retrieved rank (0-based).
    /// Value is the relevance level:
    ///   - >= 0: Document was judged and has this relevance grade.
    ///   - -1 (RELVALUE_NONPOOL): Document was retrieved but not judged (not in qrels pool).
    ///   - -2 (RELVALUE_UNJUDGED): Document was in pool but not judged (e.g. infAP).
    pub results_rel_list: Vec<i64>,

    /// Total number of retrieved documents evaluated (after optional truncation and judged-docs filtering).
    pub num_ret: usize,

    /// Total number of relevant documents for this topic in the judgments file.
    /// Specifically, count of documents with relevance judgment >= relevance_level_cutoff.
    pub num_rel: usize,

    /// Total retrieved relevant documents (relevance >= relevance_level_cutoff).
    pub num_rel_ret: usize,

    /// Total retrieved documents not present in the relevance judgments.
    pub num_nonpool: usize,

    /// Total retrieved documents present in judgments but marked unjudged (value < 0).
    pub num_unjudged_in_pool: usize,

    /// Flat array where `rel_levels[g]` stores the total number of documents in qrels with relevance grade `g`.
    /// Sized defensively as `max(max_grade + 1, relevance_level_cutoff + 1)`.
    pub rel_levels: Vec<usize>,
}

struct DocInfo {
    docno: String,
    sim: f64,
}

/// Aligns standard run results with qrels judgments for a single query.
pub fn align_query(
    run_query: Option<&RunQuery>,
    qrels_query: &QrelsQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
) -> QueryEvalState {
    let qid = qrels_query.qid.clone();

    // 1. Calculate max relevance grade and populate rel_levels counts
    let mut max_rel_grade = 0;
    for record in &qrels_query.records {
        if record.rel > max_rel_grade {
            max_rel_grade = record.rel;
        }
    }

    let rel_levels_size = std::cmp::max(max_rel_grade + 1, relevance_level_cutoff + 1) as usize;
    let mut rel_levels = vec![0; rel_levels_size];
    let mut num_rel = 0;

    for record in &qrels_query.records {
        if record.rel >= 0 {
            rel_levels[record.rel as usize] += 1;
            if record.rel >= relevance_level_cutoff {
                num_rel += 1;
            }
        }
    }

    // 2. If run query is missing, return a native empty state
    let run_records = match run_query {
        Some(rq) => &rq.records,
        None => {
            return QueryEvalState {
                qid,
                run_id: run_id.to_string(),
                results_rel_list: Vec::new(),
                num_ret: 0,
                num_rel,
                num_rel_ret: 0,
                num_nonpool: 0,
                num_unjudged_in_pool: 0,
                rel_levels,
            };
        }
    };

    // 3. Copy and sort run records by sim descending, and docno descending lexicographically
    let mut doc_infos: Vec<DocInfo> = run_records
        .iter()
        .map(|r| DocInfo {
            docno: r.docno.clone(),
            sim: r.sim,
        })
        .collect();

    doc_infos.sort_by(|a, b| {
        // Sort sim descending
        match b.sim.partial_cmp(&a.sim) {
            Some(std::cmp::Ordering::Equal) => {
                // Tie breaker: docno descending lexicographically
                b.docno.cmp(&a.docno)
            }
            Some(other) => other,
            None => std::cmp::Ordering::Equal, // Parser guarantees finite numbers, so None won't happen
        }
    });

    // 4. Truncate to max_docs_per_topic
    if doc_infos.len() > max_docs_per_topic {
        doc_infos.truncate(max_docs_per_topic);
    }

    let raw_num_ret = doc_infos.len();

    // 5. Build lookup map of qrels judgments for fast O(1) matching
    let mut qrels_map = std::collections::HashMap::with_capacity(qrels_query.records.len());
    for record in &qrels_query.records {
        qrels_map.insert(&record.docno, record.rel);
    }

    // 6. Construct results_rel_list and count metrics
    let mut results_rel_list = Vec::with_capacity(raw_num_ret);
    let mut num_rel_ret = 0;
    let mut num_nonpool = 0;
    let mut num_unjudged_in_pool = 0;

    for doc in &doc_infos {
        match qrels_map.get(&doc.docno) {
            Some(&rel) => {
                if rel < 0 {
                    results_rel_list.push(RELVALUE_UNJUDGED);
                    num_unjudged_in_pool += 1;
                } else {
                    results_rel_list.push(rel);
                    if rel >= relevance_level_cutoff {
                        num_rel_ret += 1;
                    }
                }
            }
            None => {
                results_rel_list.push(RELVALUE_NONPOOL);
                num_nonpool += 1;
            }
        }
    }

    // 7. If judged_docs_only_flag is set, throw out all unjudged docs
    if judged_docs_only_flag {
        results_rel_list.retain(|&rel| rel >= 0);
        // Recalculate num_rel_ret from the filtered set
        num_rel_ret = results_rel_list
            .iter()
            .filter(|&&rel| rel >= relevance_level_cutoff)
            .count();
        num_nonpool = 0;
        num_unjudged_in_pool = 0;
    }

    let num_ret = results_rel_list.len();

    QueryEvalState {
        qid,
        run_id: run_id.to_string(),
        results_rel_list,
        num_ret,
        num_rel,
        num_rel_ret,
        num_nonpool,
        num_unjudged_in_pool,
        rel_levels,
    }
}

/// Aligns standard run results with judgment group relevance judgments for a single query.
pub fn align_query_jg(
    run_query: Option<&RunQuery>,
    qrels_jg_query: &QrelsJGQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
) -> Vec<QueryEvalState> {
    qrels_jg_query
        .groups
        .iter()
        .map(|group| {
            let single_q = QrelsQuery {
                qid: qrels_jg_query.qid.clone(),
                records: group.records.clone(),
            };
            align_query(
                run_query,
                &single_q,
                run_id,
                relevance_level_cutoff,
                max_docs_per_topic,
                judged_docs_only_flag,
            )
        })
        .collect()
}

