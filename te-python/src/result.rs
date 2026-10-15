use std::collections::HashMap;
use pyo3::exceptions::{PyImportError, PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use te_rust::eval::EvaluationOutput;
use te_rust::metrics::MetricValue;

use crate::stats::ComparisonResult;




/// Rich evaluation result object providing mapping semantics and aggregation methods.
#[pyclass(name = "EvalResult")]
#[derive(Clone)]
pub struct EvalResult {
    /// Ordered list of topic IDs.
    pub qids: Vec<String>,
    /// Per-query metrics: qid -> (measure_name -> value).
    pub query_scores: HashMap<String, HashMap<String, MetricValue>>,
    /// Summary aggregate metrics: measure_name -> value.
    pub aggregate_scores: HashMap<String, MetricValue>,
    /// Number of queries evaluated.
    pub num_queries_evaluated: usize,
    /// Total queries in qrels.
    pub total_qrels_queries: usize,
}

impl EvalResult {
    pub fn from_output(output: EvaluationOutput) -> Self {
        let mut qids = Vec::with_capacity(output.query_results.len());
        let mut query_scores = HashMap::with_capacity(output.query_results.len());

        for q_eval in output.query_results {
            qids.push(q_eval.qid.clone());
            let mut scores = HashMap::with_capacity(q_eval.scores.len());
            for (name, val, _) in q_eval.scores {
                scores.insert(name, val);
            }
            query_scores.insert(q_eval.qid, scores);
        }

        let mut aggregate_scores = HashMap::with_capacity(output.summary_results.len());
        for s_eval in output.summary_results {
            aggregate_scores.insert(s_eval.name, s_eval.value);
        }

        Self {
            qids,
            query_scores,
            aggregate_scores,
            num_queries_evaluated: output.num_queries_evaluated,
            total_qrels_queries: output.total_qrels_queries,
        }
    }
}

fn metric_to_py(py: Python<'_>, val: &MetricValue) -> PyObject {
    match val {
        MetricValue::Float(f) => (*f).into_py(py),
        MetricValue::Integer(i) => (*i).into_py(py),
        MetricValue::Str(s) => s.as_str().into_py(py),
    }
}

#[pymethods]
impl EvalResult {
    /// Return the summary aggregate scores across all queries.
    pub fn aggregate(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for (name, val) in &self.aggregate_scores {
            dict.set_item(name, metric_to_py(py, val))?;
        }
        Ok(dict.into_any().unbind())
    }

    /// Return the per-query scores as a nested dictionary: {qid: {measure: value}}.
    pub fn per_query(&self, py: Python<'_>) -> PyResult<PyObject> {
        let root = PyDict::new_bound(py);
        for qid in &self.qids {
            if let Some(scores) = self.query_scores.get(qid) {
                let q_dict = PyDict::new_bound(py);
                for (name, val) in scores {
                    q_dict.set_item(name, metric_to_py(py, val))?;
                }
                root.set_item(qid, q_dict)?;
            }
        }
        Ok(root.into_any().unbind())
    }

    /// Convert entire result to a nested dictionary for full dict compatibility.
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.per_query(py)
    }

    /// Number of evaluated queries.
    pub fn __len__(&self) -> usize {
        self.qids.len()
    }

    /// Indexing by query ID: returns dictionary of scores for that query.
    pub fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<PyObject> {
        if let Some(scores) = self.query_scores.get(key) {
            let q_dict = PyDict::new_bound(py);
            for (name, val) in scores {
                q_dict.set_item(name, metric_to_py(py, val))?;
            }
            Ok(q_dict.into_any().unbind())
        } else {
            Err(PyKeyError::new_err(format!("Query '{}' not found in evaluation results", key)))
        }
    }


    /// Check if a query ID is present in the results.
    pub fn __contains__(&self, key: &str) -> bool {
        self.query_scores.contains_key(key)
    }

    /// Iterator over query IDs.
    pub fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::new_bound(py, &self.qids);
        let iter = list.call_method0("__iter__")?;
        Ok(iter.into_any().unbind())
    }

    /// List of evaluated query IDs.
    pub fn keys(&self) -> Vec<String> {
        self.qids.clone()
    }


    /// List of per-query metric dictionaries.
    pub fn values(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut vals = Vec::with_capacity(self.qids.len());
        for qid in &self.qids {
            vals.push(self.__getitem__(py, qid)?);
        }
        Ok(vals)
    }

    /// List of (query_id, scores_dict) pairs.
    pub fn items(&self, py: Python<'_>) -> PyResult<Vec<(String, PyObject)>> {
        let mut items = Vec::with_capacity(self.qids.len());
        for qid in &self.qids {
            items.push((qid.clone(), self.__getitem__(py, qid)?));
        }
        Ok(items)
    }

    /// Export results to a Pandas or Polars DataFrame.
    ///
    /// Parameters
    /// ----------
    /// format : str, default 'wide'
    ///     'wide': one row per query, measures as columns
    ///     'tidy' or 'long': (query_id, measure, value) triples
    #[pyo3(signature = (format = "wide"))]
    pub fn to_dataframe(&self, py: Python<'_>, format: &str) -> PyResult<PyObject> {
        let fmt = format.to_ascii_lowercase();

        let rows = PyList::empty_bound(py);

        if fmt == "wide" {
            for qid in &self.qids {
                let row = PyDict::new_bound(py);
                row.set_item("query_id", qid)?;
                if let Some(scores) = self.query_scores.get(qid) {
                    for (name, val) in scores {
                        row.set_item(name, metric_to_py(py, val))?;
                    }
                }
                rows.append(row)?;
            }
        } else if fmt == "tidy" || fmt == "long" {
            for qid in &self.qids {
                if let Some(scores) = self.query_scores.get(qid) {
                    for (name, val) in scores {
                        let row = PyDict::new_bound(py);
                        row.set_item("query_id", qid)?;
                        row.set_item("measure", name)?;
                        row.set_item("value", metric_to_py(py, val))?;
                        rows.append(row)?;
                    }
                }
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Invalid DataFrame format '{}'. Expected 'wide' or 'tidy'/'long'.",
                format
            )));
        }

        // Try importing pandas first
        if let Ok(pd) = py.import_bound("pandas") {
            let df = pd.call_method1("DataFrame", (rows,))?;
            return Ok(df.into_any().unbind());
        }

        // Try importing polars
        if let Ok(pl) = py.import_bound("polars") {
            let df = pl.call_method1("DataFrame", (rows,))?;
            return Ok(df.into_any().unbind());
        }

        Err(PyImportError::new_err(
            "pandas or polars is required for .to_dataframe(). Install via 'pip install trec-eval[pandas]' or 'pip install pandas'."
        ))
    }

    /// Return a 1D NumPy array of per-query scores for the specified measure.
    ///
    /// The array has guaranteed deterministic topic ordering matching .keys().
    #[pyo3(signature = (measure))]
    pub fn to_numpy(&self, py: Python<'_>, measure: &str) -> PyResult<PyObject> {
        let mut vals = Vec::with_capacity(self.qids.len());

        for qid in &self.qids {
            let score = if let Some(scores) = self.query_scores.get(qid) {
                if let Some(val) = scores.get(measure) {
                    match val {
                        MetricValue::Float(f) => *f,
                        MetricValue::Integer(i) => *i as f64,
                        _ => 0.0,
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };
            vals.push(score);
        }

        if let Ok(np) = py.import_bound("numpy") {
            let py_vals = PyList::new_bound(py, &vals);
            let arr = np.call_method1("array", (py_vals,))?;
            return Ok(arr.into_any().unbind());
        }

        // Fallback to pure Python list of floats if numpy is not installed
        let py_vals = PyList::new_bound(py, &vals);
        Ok(py_vals.into_any().unbind())
    }

    /// Compare this evaluation run against another evaluation run using a statistical significance test.
    ///
    /// Parameters
    /// ----------
    /// other : EvalResult
    ///     The comparison run result.
    /// measure : str, default 'map'
    ///     The measure to test across topics.
    /// test : str, default 'paired_t'
    ///     'paired_t', 'permutation', or 'bootstrap'.
    /// num_resamples : int, default 10000
    ///     Number of resamples for permutation/bootstrap tests.
    /// seed : int, optional
    ///     Optional RNG seed for reproducible permutation/bootstrap tests.
    #[pyo3(signature = (other, measure = "map", test = "paired_t", num_resamples = 10000, seed = None))]
    pub fn compare(
        &self,
        py: Python<'_>,
        other: &EvalResult,
        measure: &str,
        test: &str,
        num_resamples: usize,
        seed: Option<u64>,
    ) -> PyResult<ComparisonResult> {
        // Collect aligned score vectors across all topics
        let mut all_qids: Vec<String> = self.qids.clone();
        for qid in &other.qids {
            if !all_qids.contains(qid) {
                all_qids.push(qid.clone());
            }
        }

        let mut scores_a = Vec::with_capacity(all_qids.len());
        let mut scores_b = Vec::with_capacity(all_qids.len());

        for qid in &all_qids {
            let score_a = self
                .query_scores
                .get(qid)
                .and_then(|m| m.get(measure))
                .map(|v| match v {
                    MetricValue::Float(f) => *f,
                    MetricValue::Integer(i) => *i as f64,
                    _ => 0.0,
                })
                .unwrap_or(0.0);

            let score_b = other
                .query_scores
                .get(qid)
                .and_then(|m| m.get(measure))
                .map(|v| match v {
                    MetricValue::Float(f) => *f,
                    MetricValue::Integer(i) => *i as f64,
                    _ => 0.0,
                })
                .unwrap_or(0.0);

            scores_a.push(score_a);
            scores_b.push(score_b);
        }

        py.allow_threads(move || {
            crate::stats::run_significance_test(
                &scores_a,
                &scores_b,
                measure,
                test,
                num_resamples,
                seed,
            )
        })
    }

    fn __repr__(&self) -> String {

        format!(
            "<EvalResult: {} queries evaluated, {} aggregate measures>",
            self.num_queries_evaluated,
            self.aggregate_scores.len()
        )
    }

}
