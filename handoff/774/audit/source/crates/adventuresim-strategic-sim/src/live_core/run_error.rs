//! Run outcomes distinguish an operation failure from failure to record it.
use super::CoreLoopError;

pub type CoreLoopRunResult<T> = Result<T, CoreLoopRunError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreLoopRunError {
    #[error(transparent)]
    Operation(#[from] CoreLoopError),
    #[error("{error}; {report}")]
    FailureReport {
        #[source]
        error: CoreLoopError,
        report: FailureReportError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum FailureReportError {
    #[error("failure error state was unavailable")]
    ErrorStateUnavailable,
    #[error("failure diagnostic state was unavailable")]
    DiagnosticStateUnavailable,
    #[error(transparent)]
    Encoding(#[from] serde_json::Error),
    #[error("could not write failure diagnostic: {0}")]
    Write(#[from] std::io::Error),
}
