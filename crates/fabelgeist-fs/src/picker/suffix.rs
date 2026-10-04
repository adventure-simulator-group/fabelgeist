/// File System Access counts the leading dot in its suffix limit.
const MAX_SUFFIX_CODE_POINTS: usize = 16;

/// An admitted picker suffix, stored with its leading dot and exact spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSuffix(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSuffixViolation {
    Empty,
    InvalidCharacter,
    TrailingDot,
    TooLong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSuffixError {
    value: String,
    violation: FileSuffixViolation,
}
impl FileSuffixError {
    pub fn violation(&self) -> FileSuffixViolation {
        self.violation
    }
}
impl TryFrom<&str> for FileSuffix {
    type Error = FileSuffixError;
    fn try_from(value: &str) -> Result<Self, FileSuffixError> {
        let body = value.strip_prefix('.').unwrap_or(value);
        let mut violation = None;
        for byte in body.bytes() {
            if !byte.is_ascii_alphanumeric() && byte != b'+' && byte != b'.' {
                violation = Some(FileSuffixViolation::InvalidCharacter);
                break;
            }
        }
        let violation = if body.is_empty() {
            Some(FileSuffixViolation::Empty)
        } else if let Some(violation) = violation {
            Some(violation)
        } else if body.ends_with('.') {
            Some(FileSuffixViolation::TrailingDot)
        } else if body.len() + 1 > MAX_SUFFIX_CODE_POINTS {
            Some(FileSuffixViolation::TooLong)
        } else {
            None
        };
        match violation {
            Some(violation) => Err(FileSuffixError {
                value: value.into(),
                violation,
            }),
            None => Ok(Self(format!(".{body}"))),
        }
    }
}
impl std::fmt::Display for FileSuffixError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid picker suffix {:?}: {:?}",
            self.value, self.violation
        )
    }
}
impl std::error::Error for FileSuffixError {}

/// A nonempty, ordered suffix group. Repeated suffixes remain authored data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSuffixes(Vec<FileSuffix>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileSuffixesError;
impl TryFrom<Vec<FileSuffix>> for FileSuffixes {
    type Error = FileSuffixesError;
    fn try_from(suffixes: Vec<FileSuffix>) -> Result<Self, FileSuffixesError> {
        if suffixes.is_empty() {
            Err(FileSuffixesError)
        } else {
            Ok(Self(suffixes))
        }
    }
}
impl std::fmt::Display for FileSuffixesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a picker type requires at least one suffix")
    }
}
impl std::error::Error for FileSuffixesError {}

#[cfg(not(target_arch = "wasm32"))]
impl FileSuffixes {
    pub(super) fn native_extensions(&self) -> Vec<&str> {
        let mut extensions = Vec::with_capacity(self.0.len());
        for suffix in &self.0 {
            extensions.push(&suffix.0[1..]);
        }
        extensions
    }
}
#[cfg(target_arch = "wasm32")]
impl FileSuffixes {
    pub(super) fn browser_extensions(&self) -> js_sys::Array {
        let extensions = js_sys::Array::new();
        for suffix in &self.0 {
            extensions.push(&suffix.0.as_str().into());
        }
        extensions
    }
}

#[cfg(test)]
mod tests;
