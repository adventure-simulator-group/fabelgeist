//! Admission failures for the native fixed-arity operator signature helpers.

use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeOperatorSignatureError {
    BinaryOperator { function: String },
    Comparison { function: String },
}

impl fmt::Display for NativeOperatorSignatureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinaryOperator { function } => write!(
                formatter,
                "Compute shader must define a '{function}' function: fn {function}(a: T, b: T) -> T"
            ),
            Self::Comparison { function } => write!(
                formatter,
                "Compute shader must define a '{function}' function: fn {function}(a: T, b: T) -> bool"
            ),
        }
    }
}

impl Error for NativeOperatorSignatureError {}
