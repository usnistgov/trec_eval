use std::collections::HashMap;

use crate::eval::alignment::{align_query, align_query_jg};
use crate::eval::bootstrap::{bootstrap_ci_with_aggregator, bootstrap_mean_ci, BootstrapConfig, ConfidenceInterval};
use crate::io::{QrelsData, QrelsJGData, QrelsJGQuery, QrelsQuery, RunData, RunQuery};
use crate::metrics::{EvalConfig, EvalState, Measure, MetricValue, ValueFormat};


/// Evaluation scores for a single topic query.
#[derive(Debug, Clone)]
pub struct QueryEvaluation {
    pub qid: String,
    pub scores: Vec<(String, MetricValue, ValueFormat)>,
}

/// Evaluation score for summary averages across queries.
#[derive(Debug, Clone)]
pub struct SummaryEvaluation {
    pub name: String,
    pub value: MetricValue,
    pub format: ValueFormat,
    pub ci: Option<ConfidenceInterval>,
}

/// Complete results of an evaluation run across all topics and summary averages.
#[derive(Debug, Clone)]
pub struct EvaluationOutput {
    pub query_results: Vec<QueryEvaluation>,
    pub summary_results: Vec<SummaryEvaluation>,
    pub num_queries_evaluated: usize,
    pub total_qrels_queries: usize,
}

/// Evaluates a run against relevance judgments using the supplied active measures and configuration.
pub fn evaluate_run(
    qrels: &QrelsData,
    run: &RunData,
    measures: &[Box<dyn Measure>],
    config: &EvalConfig,
    ci_config: Option<&BootstrapConfig>,
) -> EvaluationOutput {
    let mut qrels_by_qid: HashMap<String, &QrelsQuery> = HashMap::with_capacity(qrels.queries.len());
    for qrels_q in &qrels.queries {
        qrels_by_qid.insert(qrels_q.qid.clone(), qrels_q);
    }

    let mut run_by_qid: HashMap<String, &RunQuery> = HashMap::with_capacity(run.queries.len());
    for run_q in &run.queries {
        run_by_qid.insert(run_q.qid.clone(), run_q);
    }

    // Collect topic IDs in alphabetical qid order (matching qrels ordering)
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

    let mut running_totals: Vec<Vec<MetricValue>> = measures
        .iter()
        .map(|m| m.initial_values())
        .collect();

    let enable_ci = ci_config.is_some();
    let mut query_scores_per_measure: Vec<Vec<Vec<f64>>> = if enable_ci {
        measures
            .iter()
            .map(|m| vec![Vec::with_capacity(eval_qids.len()); m.sub_metrics().len()])
            .collect()
    } else {
        Vec::new()
    };

    let mut query_results = Vec::with_capacity(if config.query_flag { eval_qids.len() } else { 0 });

    for qid in &eval_qids {
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

        let eval_state = EvalState::Standard(q_state);
        let mut q_eval_scores = Vec::new();

        for (m_idx, m) in measures.iter().enumerate() {
            let q_scores = m.calc(config, &eval_state);

            if config.query_flag && m.is_query_enabled() {
                let sub_names = m.sub_metrics();
                for (sub_idx, val) in q_scores.iter().enumerate() {
                    let name = sub_names[sub_idx].clone();
                    q_eval_scores.push((name, val.clone(), m.format()));
                }
            }

            if enable_ci && m.is_summary_enabled() && m.format() == ValueFormat::Float {
                for (sub_idx, val) in q_scores.iter().enumerate() {
                    if let MetricValue::Float(f) = val {
                        query_scores_per_measure[m_idx][sub_idx].push(*f);
                    }
                }
            }

            m.accumulate(&q_scores, &mut running_totals[m_idx]);
        }

        if config.query_flag {
            query_results.push(QueryEvaluation {
                qid: qid.clone(),
                scores: q_eval_scores,
            });
        }
    }

    let mut summary_results = Vec::new();
    let total_qrels_queries = qrels.queries.len();

    if config.summary_flag && num_queries_evaluated > 0 {
        for (m_idx, m) in measures.iter().enumerate() {
            if !m.is_summary_enabled() {
                continue;
            }
            let mut totals = running_totals[m_idx].clone();
            m.average(config, &mut totals, num_queries_evaluated, total_qrels_queries);

            let sub_names = m.sub_metrics();
            for (sub_idx, val) in totals.into_iter().enumerate() {
                let name = sub_names[sub_idx].clone();

                let ci = if let (Some(boot_config), ValueFormat::Float) = (ci_config, m.format()) {
                    let scores = &query_scores_per_measure[m_idx][sub_idx];
                    let is_geo_mean = m.name() == "gm_map" || m.name() == "gm_bpref";
                    if is_geo_mean {
                        bootstrap_ci_with_aggregator(scores, boot_config, |sample| {
                            (sample.iter().sum::<f64>() / sample.len() as f64).exp()
                        })
                    } else {
                        bootstrap_mean_ci(scores, boot_config)
                    }
                } else {
                    None
                };

                summary_results.push(SummaryEvaluation {
                    name,
                    value: val,
                    format: m.format(),
                    ci,
                });
            }
        }
    }

    EvaluationOutput {
        query_results,
        summary_results,
        num_queries_evaluated,
        total_qrels_queries,
    }
}

/// Evaluates a run against judgment group relevance judgments using the supplied active measures.
pub fn evaluate_run_jg(
    qrels: &QrelsJGData,
    run: &RunData,
    measures: &[Box<dyn Measure>],
    config: &EvalConfig,
    ci_config: Option<&BootstrapConfig>,
) -> EvaluationOutput {
    let mut qrels_by_qid: HashMap<String, &QrelsJGQuery> =
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

    let mut running_totals: Vec<Vec<MetricValue>> = measures
        .iter()
        .map(|m| m.initial_values())
        .collect();

    let enable_ci = ci_config.is_some();
    let mut query_scores_per_measure: Vec<Vec<Vec<f64>>> = if enable_ci {
        measures
            .iter()
            .map(|m| vec![Vec::with_capacity(eval_qids.len()); m.sub_metrics().len()])
            .collect()
    } else {
        Vec::new()
    };

    let mut query_results =
        Vec::with_capacity(if config.query_flag { eval_qids.len() } else { 0 });

    for qid in &eval_qids {
        let qrels_q = qrels_by_qid.get(qid).unwrap();
        let run_q = run_by_qid.get(qid).copied();

        let q_states = align_query_jg(
            run_q,
            qrels_q,
            &run.run_id,
            config.relevance_level,
            config.max_num_docs_per_topic,
            config.judged_docs_only_flag,
        );

        let eval_state = EvalState::JudgmentGroups(q_states);
        let mut q_eval_scores = Vec::new();

        for (m_idx, m) in measures.iter().enumerate() {
            let q_scores = m.calc(config, &eval_state);

            if config.query_flag && m.is_query_enabled() {
                let sub_names = m.sub_metrics();
                for (sub_idx, val) in q_scores.iter().enumerate() {
                    let name = sub_names[sub_idx].clone();
                    q_eval_scores.push((name, val.clone(), m.format()));
                }
            }

            if enable_ci && m.is_summary_enabled() && m.format() == ValueFormat::Float {
                for (sub_idx, val) in q_scores.iter().enumerate() {
                    if let MetricValue::Float(f) = val {
                        query_scores_per_measure[m_idx][sub_idx].push(*f);
                    }
                }
            }

            m.accumulate(&q_scores, &mut running_totals[m_idx]);
        }

        if config.query_flag {
            query_results.push(QueryEvaluation {
                qid: qid.clone(),
                scores: q_eval_scores,
            });
        }
    }

    let mut summary_results = Vec::new();
    let total_qrels_queries = qrels.queries.len();

    if config.summary_flag && num_queries_evaluated > 0 {
        for (m_idx, m) in measures.iter().enumerate() {
            if !m.is_summary_enabled() {
                continue;
            }
            let mut totals = running_totals[m_idx].clone();
            m.average(
                config,
                &mut totals,
                num_queries_evaluated,
                total_qrels_queries,
            );

            let sub_names = m.sub_metrics();
            for (sub_idx, val) in totals.into_iter().enumerate() {
                let name = sub_names[sub_idx].clone();

                let ci = if let (Some(boot_config), ValueFormat::Float) = (ci_config, m.format()) {
                    let scores = &query_scores_per_measure[m_idx][sub_idx];
                    let is_geo_mean = m.name() == "gm_map" || m.name() == "gm_bpref";
                    if is_geo_mean {
                        bootstrap_ci_with_aggregator(scores, boot_config, |sample| {
                            (sample.iter().sum::<f64>() / sample.len() as f64).exp()
                        })
                    } else {
                        bootstrap_mean_ci(scores, boot_config)
                    }
                } else {
                    None
                };

                summary_results.push(SummaryEvaluation {
                    name,
                    value: val,
                    format: m.format(),
                    ci,
                });
            }
        }
    }

    EvaluationOutput {
        query_results,
        summary_results,
        num_queries_evaluated,
        total_qrels_queries,
    }
}

