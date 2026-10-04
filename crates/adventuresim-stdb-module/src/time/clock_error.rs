//! Admission failure for the official strategic clock singleton.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldClockError {
    NotInitialized,
}

impl std::fmt::Display for WorldClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => f.write_str("World clock is not initialized"),
        }
    }
}

impl std::error::Error for WorldClockError {}
