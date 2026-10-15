use std::collections::HashMap;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence, PySet, PyString};
use te_rust::eval::alignment::align_query;
use te_rust::eval::{evaluate_run, EvaluationOutput, QueryEvaluation, SummaryEvaluation};
use te_rust::io::{QrelsData, QrelsQuery, RunData, RunQuery};
use te_rust::metrics::{registry::resolve_measures, EvalConfig, MetricValue, ValueFormat};

use crate::conversions::{ingest_qrels, ingest_run, ingest_run_arrays, ColumnMapping};
use crate::query_state::QueryState;
use crate::result::EvalResult;

/// Core evaluation engine holding pre-indexed relevance judgments and evaluation configuration.
#[pyclass(subclass, name = "Evaluator")]
pub struct Evaluator {
    pub qrels: QrelsData,
    pub measure_names: Vec<String>,
    pub custom_measures: Vec<PyObject>,
    pub config: EvalConfig,
}

#[pymethods]
impl Evaluator {
    #[new]
    #[pyo3(signature = (
        qrels,
        measures = None,
        relevance_level = 1,
        complete_set_average = false,
        judged_docs_only = false,
        max_docs_per_topic = None,
        num_docs_in_coll = 0,
        global_gains = None,
        qid = None,
        docno = None,
        rel = None
    ))]
    pub fn new(
        qrels: &Bound<'_, PyAny>,
        measures: Option<&Bound<'_, PyAny>>,
        relevance_level: i64,
        complete_set_average: bool,
        judged_docs_only: bool,
        max_docs_per_topic: Option<usize>,
        num_docs_in_coll: usize,
        global_gains: Option<String>,
        qid: Option<String>,
        docno: Option<String>,
        rel: Option<String>,
    ) -> PyResult<Self> {
        let mapping = ColumnMapping {
            qid,
            docno,
            score_or_rel: rel,
        };
        let qrels_data = ingest_qrels(qrels, Some(&mapping))?;

        let (measure_names, custom_measures) = parse_measures_arg(measures)?;

        // Validate built-in measure names by attempting resolution
        if !measure_names.is_empty() {
            resolve_measures(&measure_names).map_err(|e| {
                PyValueError::new_err(format!("Invalid measure configuration: {}", e))
            })?;
        }

        let parsed_gains = global_gains
            .as_ref()
            .map(|s| te_rust::metrics::common::GainsConfig::parse(s));

        let config = EvalConfig {
            query_flag: true,
            summary_flag: true,
            relevance_level,
            average_complete_flag: complete_set_average,
            judged_docs_only_flag: judged_docs_only,
            max_num_docs_per_topic: max_docs_per_topic.unwrap_or(usize::MAX),
            num_docs_in_coll,
            global_gains: parsed_gains,
        };

        Ok(Self {
            qrels: qrels_data,
            measure_names,
            custom_measures,
            config,
        })
    }


    /// Evaluate a system run against pre-indexed relevance judgments.
    #[pyo3(signature = (run, qid = None, docno = None, score = None))]
    pub fn evaluate(
        &self,
        py: Python<'_>,
        run: &Bound<'_, PyAny>,
        qid: Option<String>,
        docno: Option<String>,
        score: Option<String>,
    ) -> PyResult<EvalResult> {
        let mapping = ColumnMapping {
            qid,
            docno,
            score_or_rel: score,
        };
        let run_data = ingest_run(run, Some(&mapping))?;

        let active_measures = if !self.measure_names.is_empty() {
            resolve_measures(&self.measure_names).map_err(|e| {
                PyValueError::new_err(format!("Measure resolution error: {}", e))
            })?
        } else {
            Vec::new()
        };

        let qrels_ref = &self.qrels;
        let config_ref = &self.config;

        // 1. Run built-in measures with GIL released
        let mut output = py.allow_threads(|| {
            evaluate_run(qrels_ref, &run_data, &active_measures, config_ref, None)
        });


        // 2. Run custom measures with exception sandboxing
        if !self.custom_measures.is_empty() {
            evaluate_custom_measures(
                py,
                qrels_ref,
                &run_data,
                &self.custom_measures,
                config_ref,
                &mut output,
            )?;
        }

        Ok(EvalResult::from_output(output))
    }

    /// Evaluate 1D contiguous arrays/sequences of query IDs, doc IDs, and scores.
    pub fn evaluate_arrays(
        &self,
        py: Python<'_>,
        query_ids: &Bound<'_, PyAny>,
        doc_ids: &Bound<'_, PyAny>,
        scores: &Bound<'_, PyAny>,
    ) -> PyResult<EvalResult> {
        let run_data = ingest_run_arrays(query_ids, doc_ids, scores)?;

        let active_measures = if !self.measure_names.is_empty() {
            resolve_measures(&self.measure_names).map_err(|e| {
                PyValueError::new_err(format!("Measure resolution error: {}", e))
            })?
        } else {
            Vec::new()
        };

        let qrels_ref = &self.qrels;
        let config_ref = &self.config;

        let mut output = py.allow_threads(|| {
            evaluate_run(qrels_ref, &run_data, &active_measures, config_ref, None)
        });


        if !self.custom_measures.is_empty() {
            evaluate_custom_measures(
                py,
                qrels_ref,
                &run_data,
                &self.custom_measures,
                config_ref,
                &mut output,
            )?;
        }

        Ok(EvalResult::from_output(output))
    }
}

fn parse_measures_arg(
    measures: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Vec<String>, Vec<PyObject>)> {
    let mut names = Vec::new();
    let mut custom = Vec::new();

    let Some(obj) = measures else {
        return Ok((vec!["official".to_string()], Vec::new()));
    };

    if let Ok(py_str) = obj.downcast::<PyString>() {
        let s = py_str.to_string_lossy().into_owned();
        names.push(s);
        return Ok((names, custom));
    }

    // Check if single custom measure object
    if obj.hasattr("calc_query").unwrap_or(false) {
        custom.push(obj.clone().into_any().unbind());
        return Ok((names, custom));
    }

    if let Ok(set) = obj.downcast::<PySet>() {
        for item in set.iter() {
            if let Ok(py_str) = item.downcast::<PyString>() {
                names.push(py_str.to_string_lossy().into_owned());
            } else {
                custom.push(item.into_any().unbind());
            }
        }
        return Ok((names, custom));
    }

    if let Ok(seq) = obj.downcast::<PySequence>() {
        let len = seq.len()?;
        for i in 0..len {
            let item = seq.get_item(i)?;
            if let Ok(py_str) = item.downcast::<PyString>() {
                names.push(py_str.to_string_lossy().into_owned());
            } else {
                custom.push(item.into_any().unbind());
            }
        }
        return Ok((names, custom));
    }

    if let Ok(iter) = obj.iter() {
        for item_res in iter {
            let item = item_res?;
            if let Ok(py_str) = item.downcast::<PyString>() {
                names.push(py_str.to_string_lossy().into_owned());
            } else {
                custom.push(item.into_any().unbind());
            }
        }
        return Ok((names, custom));
    }

    Err(PyTypeError::new_err(
        "Invalid measures argument: expected string, list of strings/measures, set, or None",
    ))
}

fn evaluate_custom_measures(
    py: Python<'_>,
    qrels: &QrelsData,
    run: &RunData,
    custom_measures: &[PyObject],
    config: &EvalConfig,
    output: &mut EvaluationOutput,
) -> PyResult<()> {
    let mut qrels_by_qid: HashMap<String, &QrelsQuery> =
        HashMap::with_capacity(qrels.queries.len());
    for qrels_q in &qrels.queries {
        qrels_by_qid.insert(qrels_q.qid.clone(), qrels_q);
    }

    let mut run_by_qid: HashMap<String, &RunQuery> = HashMap::with_capacity(run.queries.len());
    for run_q in &run.queries {
        run_by_qid.insert(run_q.qid.clone(), run_q);
    }

    let mut eval_qids = Vec::new();
    let mut num_queries_evaluated = 0;

    for qrels_q in &qrels.queries {
        let in_run = run_by_qid.contains_key(&qrels_q.qid);
        if in_run {
            num_queries_evaluated += 1;
        }
        if config.average_complete_flag || in_run {
            eval_qids.push(qrels_q.qid.clone());
        }
    }

    // Extract names for custom measures
    let mut custom_names = Vec::with_capacity(custom_measures.len());
    for meas in custom_measures {
        let bound = meas.bind(py);
        let name = if let Ok(n) = bound.getattr("name") {
            n.extract::<String>()?
        } else if let Ok(n) = bound.getattr("__name__") {
            n.extract::<String>()?
        } else {
            "custom_measure".to_string()
        };
        custom_names.push(name);
    }

    let mut custom_query_scores: Vec<Vec<f64>> =
        vec![Vec::with_capacity(eval_qids.len()); custom_measures.len()];

    for (q_idx, qid) in eval_qids.iter().enumerate() {
        let qrels_q = qrels_by_qid.get(qid).unwrap();
        let run_q = run_by_qid.get(qid).copied();

        let q_state = align_query(
            run_q,
            qrels_q,
            &run.run_id,
            config.relevance_level,
            config.max_num_docs_per_topic,
            config.judged_docs_only_flag,
        );

        let py_query_state = QueryState::from_eval_state(&q_state, config.relevance_level);

        for (m_idx, meas) in custom_measures.iter().enumerate() {
            let meas_name = &custom_names[m_idx];
            let bound = meas.bind(py);

            // Calculate query score via calc_query or __call__
            let val_obj = if bound.hasattr("calc_query").unwrap_or(false) {
                bound.call_method1("calc_query", (py_query_state.clone(),))
            } else {
                bound.call1((py_query_state.clone(),))
            }
            .map_err(|e| {
                PyValueError::new_err(format!(
                    "EvaluationError in custom measure '{}' on query '{}': {}",
                    meas_name, qid, e
                ))
            })?;

            let float_val = if let Ok(f) = val_obj.extract::<f64>() {
                f
            } else if let Ok(i) = val_obj.extract::<i64>() {
                i as f64
            } else {
                return Err(PyTypeError::new_err(format!(
                    "Custom measure '{}' on query '{}' must return a float or int",
                    meas_name, qid
                )));
            };

            custom_query_scores[m_idx].push(float_val);

            // If query results exist in output, append to existing query
            if config.query_flag {
                if q_idx < output.query_results.len() {
                    output.query_results[q_idx].scores.push((
                        meas_name.clone(),
                        MetricValue::Float(float_val),
                        ValueFormat::Float,
                    ));
                } else {
                    output.query_results.push(QueryEvaluation {
                        qid: qid.clone(),
                        scores: vec![(
                            meas_name.clone(),
                            MetricValue::Float(float_val),
                            ValueFormat::Float,
                        )],
                    });
                }
            }
        }
    }

    // Compute aggregate for each custom measure
    let total_qrels_queries = qrels.queries.len();
    if config.summary_flag && num_queries_evaluated > 0 {
        for (m_idx, meas) in custom_measures.iter().enumerate() {
            let meas_name = &custom_names[m_idx];
            let scores = &custom_query_scores[m_idx];
            let bound = meas.bind(py);

            let agg_val: f64 = if bound.hasattr("aggregate").unwrap_or(false) {
                let py_scores = PyList::new_bound(py, scores);
                let agg_obj = bound
                    .call_method1(
                        "aggregate",
                        (
                            py_scores,
                            num_queries_evaluated,
                            total_qrels_queries,
                            config.average_complete_flag,
                        ),
                    )
                    .map_err(|e| {
                        PyValueError::new_err(format!(
                            "EvaluationError in custom measure '{}' aggregate(): {}",
                            meas_name, e
                        ))
                    })?;
                agg_obj.extract::<f64>()?
            } else {
                let denom = if config.average_complete_flag {
                    total_qrels_queries
                } else {
                    num_queries_evaluated
                };
                if denom > 0 {
                    scores.iter().sum::<f64>() / denom as f64
                } else {
                    0.0
                }
            };

            output.summary_results.push(SummaryEvaluation {
                name: meas_name.clone(),
                value: MetricValue::Float(agg_val),
                format: ValueFormat::Float,
                ci: None,
            });
        }
    }

    Ok(())
}

