//! Rejections from the Momentum model-definition text parser.

use std::error::Error;
use std::fmt;
use std::num::ParseFloatError;

/// A rejected header, assignment target or expression coefficient.
///
/// Text fields retain native diagnostic provenance after comment removal and
/// trimming. They do not introduce parameter or joint identity types. Other
/// malformed terms and unsupported sections retain the parser's skip policy.
#[derive(Debug)]
pub enum ModelDefinitionError {
    /// No nonempty, uncommented line supplied the version header.
    MissingHeader,
    /// The first meaningful line was not the supported version header.
    InvalidHeader { line: String },
    /// An assignment target did not contain a joint/channel separator.
    InvalidTarget { line: String, target: String },
    /// The assignment target named a joint absent from the supplied skeleton.
    UnknownJoint { line: String, joint: String },
    /// The assignment target named an unsupported joint channel.
    UnknownChannel { line: String, channel: String },
    /// The first factor of a two-factor expression was not a native float.
    InvalidExpressionCoefficient {
        line: String,
        token: String,
        source: ParseFloatError,
    },
}

impl fmt::Display for ModelDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader => {
                formatter.write_str("invalid model definition file; missing the version header")
            }
            Self::InvalidHeader { line } => {
                write!(formatter, "invalid model definition file; got {line:?}")
            }
            Self::InvalidTarget { line, .. } | Self::UnknownJoint { line, .. } => {
                write!(formatter, "unknown joint name in expression: {line}")
            }
            Self::UnknownChannel { line, .. } => {
                write!(formatter, "unknown channel name in expression: {line}")
            }
            Self::InvalidExpressionCoefficient { line, .. } => {
                write!(formatter, "could not parse weight in: {line}")
            }
        }
    }
}

impl Error for ModelDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidExpressionCoefficient { source, .. } => Some(source),
            _ => None,
        }
    }
}
