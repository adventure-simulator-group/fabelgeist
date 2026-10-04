//! Alcohol domain admission failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlcoholIntervalError {
    Reversed,
    TooLong,
    EveningOverflow,
}
impl std::fmt::Display for AlcoholIntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Reversed => "Alcohol interval ends before it starts",
            Self::TooLong => "Alcohol interval cannot exceed one year",
            Self::EveningOverflow => "Evening identity overflow",
        })
    }
}
impl std::error::Error for AlcoholIntervalError {}
