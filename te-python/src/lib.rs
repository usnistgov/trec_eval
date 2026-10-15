pub mod conversions;
pub mod evaluator;
pub mod query_state;
pub mod result;
pub mod stats;

use pyo3::prelude::*;
use evaluator::Evaluator;
use query_state::QueryState;
use result::EvalResult;
use stats::ComparisonResult;

/// Python module definition for `_trec_eval`
#[pymodule]
fn _trec_eval(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "11.0.0.dev0")?;
    m.add_class::<Evaluator>()?;
    m.add_class::<EvalResult>()?;
    m.add_class::<ComparisonResult>()?;
    m.add_class::<QueryState>()?;
    Ok(())
}



