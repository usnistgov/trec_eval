use std::collections::BTreeMap;

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

    /// Sparse map of distinct judged relevance grades (`>= 0`) in qrels to their document counts,
    /// ordered by ascending grade. Empty if all qrels for this topic have negative grades.
    pub rel_levels: BTreeMap<i64, usize>,
}

struct DocInfo {
    docno: String,
    sim: f64,
}

/// Strict total order for ranking documents: higher `sim` first, ties broken
/// by `docno` descending. Normalizes `-0.0` to `+0.0` (matching C `trec_eval`
/// tie-breaking) and places any `NaN` deterministically after all finite/infinite
/// scores so direct library callers cannot trigger non-transitive `sort_by` panics.
fn cmp_doc_info(a: &DocInfo, b: &DocInfo) -> std::cmp::Ordering {
    match (a.sim.is_nan(), b.sim.is_nan()) {
        (true, true) => b.docno.cmp(&a.docno),
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => {
            let a_sim = if a.sim == 0.0 { 0.0 } else { a.sim };
            let b_sim = if b.sim == 0.0 { 0.0 } else { b.sim };
            b_sim.total_cmp(&a_sim).then_with(|| b.docno.cmp(&a.docno))
        }
    }
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

    // 1. Populate distinct judged (>= 0) relevance grade counts and total relevant count
    let mut rel_levels: BTreeMap<i64, usize> = BTreeMap::new();
    let mut num_rel = 0;

    for record in &qrels_query.records {
        if record.rel >= 0 {
            *rel_levels.entry(record.rel).or_insert(0) += 1;
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

    doc_infos.sort_by(cmp_doc_info);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{QrelsRecord, RunRecord};

    #[test]
    fn test_align_query_signed_zero_tiebreak() {
        let qrels = QrelsQuery {
            qid: "1".to_string(),
            records: vec![
                QrelsRecord { docno: "a".to_string(), rel: 1 },
                QrelsRecord { docno: "b".to_string(), rel: 2 },
            ],
        };
        let run = RunQuery {
            qid: "1".to_string(),
            records: vec![
                RunRecord { docno: "a".to_string(), sim: 0.0 },
                RunRecord { docno: "b".to_string(), sim: -0.0 },
            ],
        };

        let state = align_query(Some(&run), &qrels, "r", 1, usize::MAX, false);
        // -0.0 and +0.0 tie, broken by docno descending: "b" (rel 2) then "a" (rel 1).
        assert_eq!(state.results_rel_list, vec![2, 1]);
    }

    #[test]
    fn test_align_query_nan_is_total_order_and_deterministic() {
        let qrels = QrelsQuery {
            qid: "1".to_string(),
            records: vec![
                QrelsRecord { docno: "a".to_string(), rel: 1 },
                QrelsRecord { docno: "b".to_string(), rel: 2 },
                QrelsRecord { docno: "c".to_string(), rel: 3 },
            ],
        };

        let orders = [
            vec![("a", 5.0), ("b", f64::NAN), ("c", 3.0)],
            vec![("c", 3.0), ("b", f64::NAN), ("a", 5.0)],
            vec![("b", f64::NAN), ("c", 3.0), ("a", 5.0)],
        ];
        for records in orders {
            let run = RunQuery {
                qid: "1".to_string(),
                records: records
                    .into_iter()
                    .map(|(docno, sim)| RunRecord {
                        docno: docno.to_string(),
                        sim,
                    })
                    .collect(),
            };
            let state = align_query(Some(&run), &qrels, "r", 1, usize::MAX, false);
            // Finite scores rank descending (a=5, c=3), NaN (b) deterministically last.
            assert_eq!(state.results_rel_list, vec![1, 3, 2]);
        }

        // Larger run with multiple NaNs must not panic slice::sort_by.
        let large_qrels = QrelsQuery {
            qid: "1".to_string(),
            records: (0..40)
                .map(|i| QrelsRecord {
                    docno: format!("d{:02}", i),
                    rel: if i < 10 { 1 } else { 0 },
                })
                .collect(),
        };
        let large_run = RunQuery {
            qid: "1".to_string(),
            records: (0..40)
                .map(|i| RunRecord {
                    docno: format!("d{:02}", i),
                    sim: if i % 7 == 0 { f64::NAN } else { (i % 13) as f64 },
                })
                .collect(),
        };
        let state = align_query(Some(&large_run), &large_qrels, "r", 1, usize::MAX, false);
        assert_eq!(state.num_ret, 40);
    }

    #[test]
    fn test_align_query_sparse_rel_levels_and_extreme_cutoffs() {
        // Sparse huge grades including i64::MAX must only store distinct judged grades.
        let qrels = QrelsQuery {
            qid: "1".to_string(),
            records: vec![
                QrelsRecord { docno: "a".to_string(), rel: 0 },
                QrelsRecord { docno: "b".to_string(), rel: 500_000_000 },
                QrelsRecord { docno: "c".to_string(), rel: i64::MAX },
                QrelsRecord { docno: "u".to_string(), rel: -2 },
            ],
        };
        let run = RunQuery {
            qid: "1".to_string(),
            records: vec![
                RunRecord { docno: "c".to_string(), sim: 3.0 },
                RunRecord { docno: "b".to_string(), sim: 2.0 },
                RunRecord { docno: "a".to_string(), sim: 1.0 },
            ],
        };

        let expected_levels = BTreeMap::from([(0, 1), (500_000_000, 1), (i64::MAX, 1)]);

        // Huge relevance_level_cutoff (1e12 and i64::MAX) must not pad rel_levels or overflow.
        let state_1e12 = align_query(Some(&run), &qrels, "r", 1_000_000_000_000, usize::MAX, false);
        assert_eq!(state_1e12.rel_levels, expected_levels);
        assert_eq!(state_1e12.num_rel, 1);
        assert_eq!(state_1e12.num_rel_ret, 1);

        let state_max = align_query(Some(&run), &qrels, "r", i64::MAX, usize::MAX, false);
        assert_eq!(state_max.rel_levels, expected_levels);
        assert_eq!(state_max.num_rel, 1);

        // Negative relevance_level_cutoff marks all judged (>= 0) docs relevant.
        let state_neg = align_query(Some(&run), &qrels, "r", -1, usize::MAX, false);
        assert_eq!(state_neg.rel_levels, expected_levels);
        assert_eq!(state_neg.num_rel, 3);
        assert_eq!(state_neg.num_rel_ret, 3);

        // All-negative topic yields empty rel_levels and zero num_rel.
        let all_neg = QrelsQuery {
            qid: "2".to_string(),
            records: vec![
                QrelsRecord { docno: "a".to_string(), rel: -1 },
                QrelsRecord { docno: "b".to_string(), rel: -2 },
            ],
        };
        let state_all_neg = align_query(Some(&run), &all_neg, "r", 1, usize::MAX, false);
        assert!(state_all_neg.rel_levels.is_empty());
        assert_eq!(state_all_neg.num_rel, 0);
        assert_eq!(state_all_neg.num_rel_ret, 0);
    }
}

