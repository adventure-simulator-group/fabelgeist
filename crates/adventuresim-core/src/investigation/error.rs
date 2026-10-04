//! Investigation record validation failures.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidId,
    TextTooLong,
    OutOfRange,
    TooManyRecords,
    DuplicateRecord,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidId => "investigation ID must be bounded canonical ASCII",
            Self::TextTooLong => "investigation text is empty or exceeds its bound",
            Self::OutOfRange => "investigation value is out of range",
            Self::TooManyRecords => "investigation record count exceeds its bound",
            Self::DuplicateRecord => "investigation record identity is duplicated",
        })
    }
}
impl std::error::Error for ValidationError {}
