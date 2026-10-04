//! Disease domain admission failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiseaseIntervalError {
    PresenceSpanBound,
    ExposureWorkBound,
}
impl std::fmt::Display for DiseaseIntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PresenceSpanBound => "Disease interval has too many raw presence spans",
            Self::ExposureWorkBound => "Disease interval exceeds bounded exposure work",
        })
    }
}
impl std::error::Error for DiseaseIntervalError {}
