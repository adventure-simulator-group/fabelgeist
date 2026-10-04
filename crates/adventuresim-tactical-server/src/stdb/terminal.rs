//! Admission of terminal reducer replies and queue failures at the SDK boundary.
//!
//! Queue success precedes strategic acceptance. A reducer rejection and a failed
//! SDK callback both permit the existing retry policy, but retain distinct
//! causes until presentation. The remote reducer's message is opaque protocol
//! data; it never determines a local error classification.

use adventuresim_stdb_client::TacticalMissionResolution;
use spacetimedb_sdk::__codegen::InternalError;
use std::{error::Error, fmt};

#[derive(Clone, Debug)]
pub(crate) enum TerminalSubmissionResult {
    Accepted,
    Rejected(TerminalSubmissionError),
}

impl From<Result<Result<(), String>, InternalError>> for TerminalSubmissionResult {
    fn from(reply: Result<Result<(), String>, InternalError>) -> Self {
        match reply {
            Ok(Ok(())) => Self::Accepted,
            Ok(Err(message)) => {
                Self::Rejected(TerminalSubmissionError::ReducerRejected(message.into()))
            }
            Err(source) => Self::Rejected(TerminalSubmissionError::CallbackFailed(source)),
        }
    }
}

/// Exact rejection text supplied by the strategic reducer's wire protocol.
#[derive(Clone, Debug)]
pub(crate) struct TerminalReducerRejection(String);

impl From<String> for TerminalReducerRejection {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl fmt::Display for TerminalReducerRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for TerminalReducerRejection {}

#[derive(Clone, Debug)]
pub(crate) enum TerminalSubmissionError {
    ReducerRejected(TerminalReducerRejection),
    CallbackFailed(InternalError),
}

impl fmt::Display for TerminalSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReducerRejected(reason) => reason.fmt(formatter),
            Self::CallbackFailed(source) => {
                write!(formatter, "internal reducer callback error: {source:?}")
            }
        }
    }
}

impl Error for TerminalSubmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReducerRejected(reason) => Some(reason),
            Self::CallbackFailed(source) => Some(source),
        }
    }
}

/// Synchronous failure to enqueue one resolution for strategic acceptance.
#[derive(Debug)]
pub(crate) struct TerminalEnqueueError {
    resolution: TacticalMissionResolution,
    source: spacetimedb_sdk::Error,
}

impl TerminalEnqueueError {
    pub(super) fn from_provider(
        resolution: TacticalMissionResolution,
        source: spacetimedb_sdk::Error,
    ) -> Self {
        Self { resolution, source }
    }
}

impl fmt::Display for TerminalEnqueueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to enqueue {:?} terminal submission: {}",
            self.resolution, self.source
        )
    }
}

impl Error for TerminalEnqueueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceptance_requires_both_callback_and_reducer_success() {
        assert!(matches!(
            TerminalSubmissionResult::from(Ok(Ok(()))),
            TerminalSubmissionResult::Accepted
        ));
    }

    #[test]
    fn reducer_rejection_retains_exact_opaque_protocol_text() {
        for message in ["", "Échec: rejected\nwithout mutation"] {
            let result = TerminalSubmissionResult::from(Ok(Err(message.into())));
            let TerminalSubmissionResult::Rejected(error) = result else {
                panic!("a reducer rejection must not be accepted");
            };
            assert!(matches!(error, TerminalSubmissionError::ReducerRejected(_)));
            assert_eq!(error.to_string(), message);
            let source = error
                .source()
                .unwrap()
                .downcast_ref::<TerminalReducerRejection>()
                .unwrap();
            assert_eq!(source.to_string(), message);
        }
    }

    #[test]
    fn callback_failure_retains_the_sdk_cause_and_existing_diagnostic() {
        let source = InternalError::failed_parse("TerminalReply", "EndTacticalServer").with_cause(
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed callback payload",
            ),
        );
        let diagnostic = format!("internal reducer callback error: {source:?}");
        let result = TerminalSubmissionResult::from(Err(source));
        let TerminalSubmissionResult::Rejected(error) = result else {
            panic!("a failed callback must not be accepted");
        };
        assert!(matches!(error, TerminalSubmissionError::CallbackFailed(_)));
        assert!(error.source().unwrap().is::<InternalError>());
        assert_eq!(error.to_string(), diagnostic);
    }

    #[test]
    fn enqueue_failure_retains_resolution_and_provider_classification() {
        let error = TerminalEnqueueError::from_provider(
            TacticalMissionResolution::Failed,
            spacetimedb_sdk::Error::Disconnected,
        );
        assert_eq!(error.resolution, TacticalMissionResolution::Failed);
        assert!(matches!(
            error
                .source()
                .unwrap()
                .downcast_ref::<spacetimedb_sdk::Error>(),
            Some(spacetimedb_sdk::Error::Disconnected)
        ));
    }
}
