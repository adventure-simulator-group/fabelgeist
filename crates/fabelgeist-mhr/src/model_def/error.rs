use super::lexical::{DefinitionLine, RejectedDefinitionToken};
use super::limit::ParameterBoundsError;
use fabelgeist_rig::RigJointName;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionNumberRole {
    ExpressionWeight,
    Minimum,
    Maximum,
    SolverWeight,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionLayout {
    JointRows,
    ParameterColumns,
    DenseTransform,
}
#[derive(Debug)]
pub enum ModelDefinitionError {
    MissingHeader,
    InvalidHeader(DefinitionLine),
    InvalidTarget(DefinitionLine),
    UnknownJoint {
        line: DefinitionLine,
        joint: RigJointName,
    },
    UnknownChannel {
        line: DefinitionLine,
        token: RejectedDefinitionToken,
    },
    Number {
        line: DefinitionLine,
        role: DefinitionNumberRole,
        token: RejectedDefinitionToken,
        source: std::num::ParseFloatError,
    },
    LimitSyntax(DefinitionLine),
    LimitBounds {
        line: DefinitionLine,
        source: ParameterBoundsError,
    },
    LayoutOverflow(DefinitionLayout),
}
impl std::fmt::Display for ModelDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeader => f.write_str("missing Momentum model definition version header"),
            Self::InvalidHeader(line) => write!(f, "invalid model definition header at {line}"),
            Self::InvalidTarget(line) => write!(f, "expected joint.channel target at {line}"),
            Self::UnknownJoint { line, joint } => write!(f, "unknown joint {joint} at {line}"),
            Self::UnknownChannel { line, token } => {
                write!(f, "unknown joint channel {token} at {line}")
            }
            Self::Number {
                line,
                role,
                token,
                source,
            } => write!(f, "invalid {role:?} {token} at {line}: {source}"),
            Self::LimitSyntax(line) => write!(f, "expected two minmax bounds at {line}"),
            Self::LimitBounds { line, source } => {
                write!(f, "invalid minmax bounds at {line}: {source}")
            }
            Self::LayoutOverflow(layout) => {
                write!(f, "model definition {layout:?} overflows storage")
            }
        }
    }
}
impl std::error::Error for ModelDefinitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Number { source, .. } => Some(source),
            Self::LimitBounds { source, .. } => Some(source),
            _ => None,
        }
    }
}
