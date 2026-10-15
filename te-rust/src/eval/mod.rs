pub mod alignment;
pub mod bootstrap;
pub mod engine;
pub mod significance;

pub use alignment::{align_query, align_query_jg, QueryEvalState};

pub use bootstrap::{bootstrap_mean_ci, bootstrap_ci_with_aggregator, BootstrapConfig, ConfidenceInterval};
pub use engine::{evaluate_run, evaluate_run_jg, EvaluationOutput, QueryEvaluation, SummaryEvaluation};

pub use significance::{
    bootstrap_test, paired_t_test, permutation_test, SignificanceTestResult,
};


