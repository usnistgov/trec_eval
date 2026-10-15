use clap::Parser;
use std::process;

use te_rust::eval::bootstrap::BootstrapConfig;
use te_rust::eval::{evaluate_run, evaluate_run_jg};
use te_rust::io::{parse_trec_qrels, parse_trec_qrels_jg, parse_trec_run};
use te_rust::metrics::{EvalConfig, MetricValue, ValueFormat};
use te_rust::metrics;




#[derive(Parser, Debug)]
#[command(
    name = "te-rust",
    version = "0.1.0",
    about = "Rust clean-room implementation of trec_eval"
)]
struct Args {
    /// In addition to summary evaluation, print evaluation for each query/topic
    #[arg(short = 'q')]
    query_flag: bool,

    /// Print no summary averages/totals at the end
    #[arg(short = 'n')]
    no_summary_flag: bool,

    /// Minimum relevance level at which a document is considered relevant
    #[arg(short = 'l', default_value = "1")]
    relevance_level: i64,

    /// Average over the complete set of queries in qrels instead of the intersection
    #[arg(short = 'c')]
    complete_set_average: bool,

    /// Remove all unjudged documents from the retrieved set before evaluating
    #[arg(short = 'J')]
    judged_docs_only: bool,

    /// Max number of documents per topic to evaluate
    #[arg(short = 'M')]
    max_docs_per_topic: Option<usize>,

    /// Number of documents in the collection
    #[arg(short = 'N', default_value = "0")]
    num_docs_in_coll: usize,

    /// Calculate only the indicated measure(s)
    #[arg(short = 'm', value_name = "measure")]
    measures: Vec<String>,

    /// Compute and print bootstrap confidence intervals for summary averages
    #[arg(short = 'C', long = "ci")]
    ci: bool,

    /// Format confidence intervals in bracketed human-readable format [lower, upper]
    #[arg(long = "ci-pretty")]
    ci_pretty: bool,

    /// Significance level alpha for confidence intervals (default: 0.05 for 95% CI)
    #[arg(long = "ci-alpha", default_value = "0.05")]
    ci_alpha: f64,

    /// Number of bootstrap resamples (default: 1000)
    #[arg(long = "ci-samples", default_value = "1000")]
    ci_samples: usize,

    /// Optional RNG seed for deterministic bootstrap resampling
    #[arg(long = "seed")]
    seed: Option<u64>,

    /// List all available measures with a short description and exit
    #[arg(long = "help-measures")]
    help_measures: bool,

    /// Print detailed explanation of the specified measure and exit
    #[arg(long = "help-measure", value_name = "name")]
    help_measure: Option<String>,

    /// Format of the relevance judgments file (e.g. qrels, qrels_jg, prefs, qrels_prefs)
    #[arg(short = 'R', long = "rel-format", default_value = "qrels")]
    rel_format: String,

    /// Path to relevance judgments (qrels) file
    qrels_file: Option<String>,


    /// Path to run results file
    run_file: Option<String>,

    /// Global relevance-to-gain mapping (e.g. --global-gains 1=3.5,2=9.0)
    #[arg(long = "global-gains", value_name = "gains")]
    global_gains: Option<String>,
}

fn handle_help_flags(help_measures: bool, help_measure: Option<&str>) {
    if help_measures {
        println!("{:<22} {:<13} {}", "Measure", "Status", "Description");
        println!("{}", "-".repeat(70));
        for spec in metrics::registry::registry() {
            if let Ok(m) = (spec.factory)("") {
                println!("{:<22} {:<13} {}", m.name(), spec.status.label(), m.short_description());
            }
        }
        println!("\nRequest measures with -m <name> (repeatable). See --help-measure <name> for details.");
        process::exit(0);
    }

    if let Some(target) = help_measure {
        for spec in metrics::registry::registry() {
            if spec.name.eq_ignore_ascii_case(target) {
                if let Ok(m) = (spec.factory)("") {
                    println!("{}", m.name());
                    if !spec.status.label().is_empty() {
                        println!("Status: {}", spec.status.label());
                    }
                    println!();
                    println!("{}", m.explanation());
                    println!();
                    // How to request it, and what parameters it accepts.
                    if spec.usage.is_empty() {
                        println!("Usage:  -m {}", m.name());
                    } else {
                        println!("Usage:  -m {}", spec.usage);
                    }
                    process::exit(0);
                }
            }
        }
        eprintln!("te-rust: Unknown measure '{}'", target);
        process::exit(1);
    }
}

fn main() {
    let args = Args::parse();

    // 1. Handle help flags if present
    if args.help_measures || args.help_measure.is_some() {
        handle_help_flags(args.help_measures, args.help_measure.as_deref());
    }

    // 2. Validate positional files
    let qrels_filename = match args.qrels_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <QRELS_FILE>\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(1);
        }
    };

    let run_filename = match args.run_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(1);
        }
    };

    // 3. Ingest run results
    let run_data = match parse_trec_run(&run_filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing run file '{}': {}", run_filename, e);
            process::exit(1);
        }
    };

    // 4. Resolve measures (groups + parameters) via the central registry.
    let default_group = match args.rel_format.to_ascii_lowercase().as_str() {
        "qrels_jg" => "qrels_jg",
        _ => "official",
    };

    let requested_names: Vec<String> = if args.measures.is_empty() {
        vec![default_group.to_string()]
    } else {
        args.measures.clone()
    };

    let active_measures = match metrics::registry::resolve_measures(&requested_names) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("te-rust: {}", e);
            process::exit(1);
        }
    };

    // 5. Setup runtime config
    let global_gains = args.global_gains.as_ref().map(|s| crate::metrics::common::GainsConfig::parse(s));
    let config = EvalConfig {
        query_flag: args.query_flag,
        summary_flag: !args.no_summary_flag,
        relevance_level: args.relevance_level,
        average_complete_flag: args.complete_set_average,
        judged_docs_only_flag: args.judged_docs_only,
        max_num_docs_per_topic: args.max_docs_per_topic.unwrap_or(usize::MAX),
        num_docs_in_coll: args.num_docs_in_coll,
        global_gains,
    };

    let enable_ci = args.ci || args.ci_pretty;
    let ci_config = if enable_ci {
        Some(BootstrapConfig {
            num_samples: args.ci_samples,
            alpha: args.ci_alpha,
            seed: args.seed,
        })
    } else {
        None
    };

    // 6. Run evaluation engine based on relevance judgment format
    let output = match args.rel_format.to_ascii_lowercase().as_str() {
        "qrels_jg" => {
            let qrels_jg_data = match parse_trec_qrels_jg(&qrels_filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error parsing qrels_jg file '{}': {}", qrels_filename, e);
                    process::exit(1);
                }
            };
            evaluate_run_jg(&qrels_jg_data, &run_data, &active_measures, &config, ci_config.as_ref())
        }
        "qrels" => {
            let qrels_data = match parse_trec_qrels(&qrels_filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error parsing qrels file '{}': {}", qrels_filename, e);
                    process::exit(1);
                }
            };
            evaluate_run(&qrels_data, &run_data, &active_measures, &config, ci_config.as_ref())
        }
        other => {
            eprintln!("te-rust: Unsupported relevance judgments format '{}'", other);
            process::exit(1);
        }
    };


    // 7. Print query results
    if config.query_flag {
        for q_eval in &output.query_results {
            for (name, val, format) in &q_eval.scores {
                print_metric_value(name, &q_eval.qid, val, *format);
            }
        }
    }

    // 8. Print summary results
    if config.summary_flag && output.num_queries_evaluated > 0 {
        for s_eval in &output.summary_results {
            if let (Some(ci), true) = (&s_eval.ci, args.ci_pretty) {
                if let MetricValue::Float(f) = &s_eval.value {
                    println!("{:<22}\t{}\t{:.4} [{:.4}, {:.4}]", s_eval.name, "all", f, ci.lower, ci.upper);
                }
            } else {
                print_metric_value(&s_eval.name, "all", &s_eval.value, s_eval.format);
                if let Some(ci) = &s_eval.ci {
                    let lower_name = format!("{}_ci_lower", s_eval.name);
                    let upper_name = format!("{}_ci_upper", s_eval.name);
                    print_metric_value(&lower_name, "all", &MetricValue::Float(ci.lower), ValueFormat::Float);
                    print_metric_value(&upper_name, "all", &MetricValue::Float(ci.upper), ValueFormat::Float);
                }
            }
        }
    }
}


fn print_metric_value(name: &str, qid: &str, val: &MetricValue, format: ValueFormat) {
    match (val, format) {
        (MetricValue::Float(f), ValueFormat::Float) => {
            println!("{:<22}\t{}\t{:.4}", name, qid, f);
        }
        (MetricValue::Integer(i), ValueFormat::Integer) => {
            println!("{:<22}\t{}\t{}", name, qid, i);
        }
        (MetricValue::Str(s), ValueFormat::Str) => {
            println!("{:<22}\t{}\t{}", name, qid, s);
        }
        (MetricValue::Str(s), ValueFormat::QuotedStr) => {
            println!("{:<22}\t{}\t'{}'", name, qid, s);
        }
        _ => {
            // Fallback
            match val {
                MetricValue::Float(f) => println!("{:<22}\t{}\t{:.4}", name, qid, f),
                MetricValue::Integer(i) => println!("{:<22}\t{}\t{}", name, qid, i),
                MetricValue::Str(s) => println!("{:<22}\t{}\t{}", name, qid, s),
            }
        }
    }
}
