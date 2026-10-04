//! Failures retain their authority, operation, and concrete diagnostic cause.
use super::{cardinality::QueryRowCount, sats::SatsQueryDecodeError};
use crate::spacetimedb::{projection::ViewProjectionError, queries::SqlQuery};
use adventuresim_core::{reducer_error::ReducerErrorCode, strategic_place::PlaceIdentityError};
use reqwest::StatusCode;
use std::{error::Error, fmt};

#[derive(Debug, thiserror::Error)]
pub(crate) enum SpacetimeError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("SQL response JSON error: {0}")]
    QueryResponseDecode(#[source] serde_json::Error),
    #[error("SpacetimeDB error: {0}")]
    Remote(#[from] RemoteDatabaseFailure),
    #[error(transparent)]
    Sats(#[from] SatsQueryDecodeError),
    #[error(transparent)]
    Cardinality(#[from] SingleRowQueryError),
    #[error("generated-row presentation conversion failed: {0}")]
    Projection(#[from] ViewProjectionError),
    #[error("generated case-site identity failed validation: {0}")]
    CaseSiteIdentity(#[source] PlaceIdentityError),
}

impl SpacetimeError {
    pub(crate) fn reducer_code(&self) -> Option<ReducerErrorCode> {
        match self {
            Self::Remote(failure) => failure.reducer_code(),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DatabaseOperation {
    Query,
    Reducer,
}

impl fmt::Display for DatabaseOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Query => "SQL query",
            Self::Reducer => "reducer call",
        })
    }
}

#[derive(Debug)]
enum RemoteDiagnostic {
    Received {
        message: String,
        reducer_code: Option<ReducerErrorCode>,
    },
    Unreadable(reqwest::Error),
}

#[derive(Debug)]
pub(crate) struct RemoteDatabaseFailure {
    operation: DatabaseOperation,
    status: StatusCode,
    diagnostic: RemoteDiagnostic,
}

impl RemoteDatabaseFailure {
    pub(crate) fn from_response(
        operation: DatabaseOperation,
        status: StatusCode,
        body: std::result::Result<String, reqwest::Error>,
    ) -> Self {
        let diagnostic = match body {
            Ok(message) => RemoteDiagnostic::Received {
                reducer_code: match operation {
                    DatabaseOperation::Reducer => {
                        adventuresim_core::reducer_error::parse_reducer_error(&message)
                    }
                    DatabaseOperation::Query => None,
                },
                message,
            },
            Err(source) => RemoteDiagnostic::Unreadable(source),
        };
        Self {
            operation,
            status,
            diagnostic,
        }
    }

    fn reducer_code(&self) -> Option<ReducerErrorCode> {
        match &self.diagnostic {
            RemoteDiagnostic::Received { reducer_code, .. } => *reducer_code,
            RemoteDiagnostic::Unreadable(_) => None,
        }
    }
}

impl fmt::Display for RemoteDatabaseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} rejected (HTTP {}): ",
            self.operation, self.status
        )?;
        match &self.diagnostic {
            RemoteDiagnostic::Received { message, .. } => formatter.write_str(message),
            RemoteDiagnostic::Unreadable(source) => {
                write!(formatter, "cannot read response: {source}")
            }
        }
    }
}

impl Error for RemoteDatabaseFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.diagnostic {
            RemoteDiagnostic::Unreadable(source) => Some(source),
            RemoteDiagnostic::Received { .. } => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("query expected at most one row but returned {received}: {query}")]
pub(crate) struct SingleRowQueryError {
    query: SqlQuery,
    received: QueryRowCount,
}

impl SingleRowQueryError {
    pub(super) fn from_query(query: SqlQuery, received: QueryRowCount) -> Self {
        Self { query, received }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_reducer_codes_are_admitted_once_and_do_not_classify_sql_failures() {
        for detail in ["contact departed", "different wording", "Unicode: 雪\n"] {
            let message = adventuresim_core::reducer_error::coded_reducer_error(
                ReducerErrorCode::DialogueContactUnavailable,
                detail,
            );
            for operation in [DatabaseOperation::Query, DatabaseOperation::Reducer] {
                let failure = RemoteDatabaseFailure::from_response(
                    operation,
                    StatusCode::CONFLICT,
                    Ok(message.clone()),
                );
                assert_eq!(failure.operation, operation);
                assert_eq!(failure.status, StatusCode::CONFLICT);
                let RemoteDiagnostic::Received {
                    message: admitted, ..
                } = &failure.diagnostic
                else {
                    panic!("expected the original remote body");
                };
                assert_eq!(admitted, &message);
                let error = SpacetimeError::Remote(failure);
                assert_eq!(
                    error.reducer_code(),
                    match operation {
                        DatabaseOperation::Reducer =>
                            Some(ReducerErrorCode::DialogueContactUnavailable),
                        DatabaseOperation::Query => None,
                    }
                );
            }
        }
    }

    #[test]
    fn remote_bodies_remain_opaque_and_unreadable_bodies_retain_the_native_cause() {
        for message in ["", "uncoded rejection", "雪\n"] {
            let failure = RemoteDatabaseFailure::from_response(
                DatabaseOperation::Reducer,
                StatusCode::BAD_REQUEST,
                Ok(message.to_owned()),
            );
            assert!(failure.to_string().ends_with(message));
            assert_eq!(failure.reducer_code(), None);
        }
        let source = reqwest::Client::new()
            .get("://invalid-url")
            .build()
            .unwrap_err();
        let failure = RemoteDatabaseFailure::from_response(
            DatabaseOperation::Query,
            StatusCode::BAD_GATEWAY,
            Err(source),
        );
        assert_eq!(failure.status, StatusCode::BAD_GATEWAY);
        assert!(
            failure
                .source()
                .unwrap()
                .downcast_ref::<reqwest::Error>()
                .unwrap()
                .is_builder()
        );
        let error = SpacetimeError::Remote(failure);
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<RemoteDatabaseFailure>()
                .is_some()
        );
        assert_eq!(error.reducer_code(), None);
    }
}
