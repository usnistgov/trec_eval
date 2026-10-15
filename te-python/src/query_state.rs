use pyo3::prelude::*;
use te_rust::eval::QueryEvalState;

/// Aligned query evaluation state passed to custom Python measures.
#[pyclass(name = "QueryState")]
#[derive(Debug, Clone)]
pub struct QueryState {
    /// Topic query ID.
    #[pyo3(get)]
    pub qid: String,
    /// Aligned list of relevance judgments for retrieved documents in ranked order.
    /// Values >= 0 are judged relevance grades; -1 is non-pool, -2 is unjudged in pool.
    #[pyo3(get)]
    pub relevances: Vec<i64>,
    /// Total relevant documents in qrels for this query.
    #[pyo3(get)]
    pub num_rel: usize,
    /// Total retrieved documents evaluated for this query.
    #[pyo3(get)]
    pub num_ret: usize,
    /// Total retrieved documents that are relevant.
    #[pyo3(get)]
    pub num_rel_ret: usize,
    /// Total retrieved documents not present in the judgment pool (-1).
    #[pyo3(get)]
    pub num_nonpool: usize,
    /// Total retrieved documents marked unjudged in pool (-2).
    #[pyo3(get)]
    pub num_unjudged_in_pool: usize,
    /// Distribution of relevance grades in qrels (index = grade, value = document count).
    #[pyo3(get)]
    pub rel_levels: Vec<usize>,
    /// Minimum relevance level cutoff configured for this evaluation (-l).
    #[pyo3(get)]
    pub relevance_level: i64,
}

impl QueryState {
    pub fn from_eval_state(state: &QueryEvalState, relevance_level: i64) -> Self {
        Self {
            qid: state.qid.clone(),
            relevances: state.results_rel_list.clone(),
            num_rel: state.num_rel,
            num_ret: state.num_ret,
            num_rel_ret: state.num_rel_ret,
            num_nonpool: state.num_nonpool,
            num_unjudged_in_pool: state.num_unjudged_in_pool,
            rel_levels: state.rel_levels.clone(),
            relevance_level,
        }
    }
}

#[pymethods]
impl QueryState {
    fn __repr__(&self) -> String {
        format!(
            "<QueryState: qid='{}', num_ret={}, num_rel={}, num_rel_ret={}>",
            self.qid, self.num_ret, self.num_rel, self.num_rel_ret
        )
    }
}
