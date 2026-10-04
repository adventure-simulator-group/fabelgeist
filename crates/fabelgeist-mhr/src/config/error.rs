use std::num::ParseIntError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterLodViolation {
    InvalidEncoding,
    UnsupportedDetail,
}
#[derive(Debug)]
enum CharacterLodFailure {
    Encoding(ParseIntError),
    Unsupported,
}
#[derive(Debug)]
pub struct CharacterLodError {
    input: String,
    failure: CharacterLodFailure,
}
impl CharacterLodError {
    pub(super) fn unsupported(value: u8) -> Self {
        Self {
            input: value.to_string(),
            failure: CharacterLodFailure::Unsupported,
        }
    }
    pub(super) fn invalid_encoding(value: &str, source: ParseIntError) -> Self {
        Self {
            input: value.to_owned(),
            failure: CharacterLodFailure::Encoding(source),
        }
    }
    pub(super) fn from_spelling(value: &str, source: Self) -> Self {
        Self {
            input: value.to_owned(),
            failure: source.failure,
        }
    }
    pub fn violation(&self) -> CharacterLodViolation {
        match self.failure {
            CharacterLodFailure::Encoding(_) => CharacterLodViolation::InvalidEncoding,
            CharacterLodFailure::Unsupported => CharacterLodViolation::UnsupportedDetail,
        }
    }
}
impl std::fmt::Display for CharacterLodError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.failure {
            CharacterLodFailure::Encoding(source) => {
                write!(
                    formatter,
                    "invalid character detail {:?}: {source}",
                    self.input
                )
            }
            CharacterLodFailure::Unsupported => write!(
                formatter,
                "character detail {} is unsupported; expected 4, 5, or 6",
                self.input
            ),
        }
    }
}
impl std::error::Error for CharacterLodError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            CharacterLodFailure::Encoding(source) => Some(source),
            CharacterLodFailure::Unsupported => None,
        }
    }
}
