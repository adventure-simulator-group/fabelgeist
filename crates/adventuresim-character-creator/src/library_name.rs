//! The name an entry is saved under in one of the studio's libraries.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A library entry's name: non-empty, without surrounding space.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LibraryName(String);

impl TryFrom<String> for LibraryName {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("a library entry needs a name".into());
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<LibraryName> for String {
    fn from(name: LibraryName) -> Self {
        name.0
    }
}

impl AsRef<str> for LibraryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LibraryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_never_blank() {
        let name = LibraryName::try_from("  Gilded vine ".to_owned()).unwrap();
        assert_eq!(name.to_string(), "Gilded vine");
        assert!(LibraryName::try_from("   ".to_owned()).is_err());
        assert!(serde_json::from_str::<LibraryName>("\"\"").is_err());
    }
}
