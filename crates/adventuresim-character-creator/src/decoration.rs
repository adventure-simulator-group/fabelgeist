//! Decoration of plate steel: the engraving cut into a piece and the trim
//! along its edges, and the named library the studio saves them to.

use fabelgeist_armor::{engraving::Engraving, trim::Trim};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub use crate::library_name::LibraryName;

mod error;
pub use error::{DecorationError, DecorationLibraryError};

/// How a plate-steel piece is decorated. Both parts are optional; a plain
/// piece has neither.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decoration {
    /// Ornament cut into the piece's metal.
    pub engraving: Option<Engraving>,
    /// A band finished apart along the piece's edges.
    pub trim: Option<Trim>,
}

impl Decoration {
    pub fn is_plain(&self) -> bool {
        self.engraving.is_none() && self.trim.is_none()
    }

    pub fn validate(&self) -> Result<(), DecorationError> {
        self.engraving
            .as_ref()
            .map_or(Ok(()), Engraving::validate)?;
        Ok(self.trim.as_ref().map_or(Ok(()), Trim::validate)?)
    }
}

/// Named decorations, designed in the studio's armory and chosen for
/// inventory articles. Choosing one copies it into the article, so a recipe
/// never depends on the library.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DecorationLibrary {
    entries: BTreeMap<LibraryName, Decoration>,
}

impl DecorationLibrary {
    /// The library saved at `path`; a library not yet saved there is empty.
    pub fn load(path: &Path) -> Result<Self, DecorationLibraryError> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(source) => {
                return Err(DecorationLibraryError::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        let library: Self = match serde_json::from_slice(&bytes) {
            Ok(library) => library,
            Err(source) => {
                return Err(DecorationLibraryError::Decode {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        library.validate()?;
        Ok(library)
    }

    pub fn save(&self, path: &Path) -> Result<(), DecorationLibraryError> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
            && let Err(source) = std::fs::create_dir_all(parent)
        {
            return Err(DecorationLibraryError::Directory {
                path: parent.to_owned(),
                source,
            });
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(DecorationLibraryError::Encode)?;
        match std::fs::write(path, bytes) {
            Ok(()) => Ok(()),
            Err(source) => Err(DecorationLibraryError::Write {
                path: path.to_owned(),
                source,
            }),
        }
    }

    fn validate(&self) -> Result<(), DecorationLibraryError> {
        for (name, decoration) in &self.entries {
            if decoration.is_plain() {
                return Err(DecorationLibraryError::Plain { name: name.clone() });
            }
            if let Err(source) = decoration.validate() {
                return Err(DecorationLibraryError::Invalid {
                    name: name.clone(),
                    source: Box::new(source),
                });
            }
        }
        Ok(())
    }

    /// Save `decoration` under `name`, replacing any decoration named so.
    pub fn insert(
        &mut self,
        name: LibraryName,
        decoration: Decoration,
    ) -> Result<(), DecorationLibraryError> {
        if decoration.is_plain() {
            return Err(DecorationLibraryError::Plain { name });
        }
        if let Err(source) = decoration.validate() {
            return Err(DecorationLibraryError::Invalid {
                name,
                source: Box::new(source),
            });
        }
        self.entries.insert(name, decoration);
        Ok(())
    }

    pub fn remove(&mut self, name: &LibraryName) -> Option<Decoration> {
        self.entries.remove(name)
    }

    pub fn get(&self, name: &LibraryName) -> Option<&Decoration> {
        self.entries.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&LibraryName, &Decoration)> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The name of a saved decoration identical to `decoration`.
    pub fn name_of(&self, decoration: &Decoration) -> Option<&LibraryName> {
        self.entries
            .iter()
            .find_map(|(name, saved)| (saved == decoration).then_some(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::ornament::Ornament;

    fn name(name: &str) -> LibraryName {
        LibraryName::try_from(name.to_owned()).unwrap()
    }

    fn vine() -> Decoration {
        Decoration {
            engraving: Some(Engraving::ornament(Ornament::default())),
            trim: Some(Trim::default()),
        }
    }

    #[test]
    fn a_saved_library_reloads_and_recognizes_its_decorations() {
        let dir = std::env::temp_dir().join(format!("decorations-{}", std::process::id()));
        let path = dir.join("nested").join("decorations.json");
        assert!(DecorationLibrary::load(&path).unwrap().is_empty());

        let mut library = DecorationLibrary::default();
        library.insert(name("Gilded vine"), vine()).unwrap();
        library.save(&path).unwrap();
        let loaded = DecorationLibrary::load(&path).unwrap();
        assert_eq!(loaded, library);
        assert_eq!(loaded.name_of(&vine()), Some(&name("Gilded vine")));
        assert_eq!(loaded.name_of(&Decoration::default()), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn plain_and_invalid_decorations_are_not_saved() {
        let mut library = DecorationLibrary::default();
        assert!(
            matches!(library.insert(name("Bare"), Decoration::default()),
            Err(DecorationLibraryError::Plain { name: rejected }) if rejected == name("Bare"))
        );
        let mut broken = vine();
        broken.trim.as_mut().unwrap().width = f32::NAN;
        assert!(matches!(library.insert(name("Broken"), broken),
            Err(DecorationLibraryError::Invalid { name: rejected, source })
                if rejected == name("Broken") && matches!(*source, DecorationError::Trim(fabelgeist_armor::trim::TrimFinishError::Width))));
        assert!(library.is_empty());
    }

    #[test]
    fn library_decode_and_io_failures_are_distinguishable() {
        use std::error::Error;

        let dir = std::env::temp_dir().join(format!("decoration-errors-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("invalid.json");
        std::fs::write(&path, b"not json").unwrap();
        let error = DecorationLibrary::load(&path).unwrap_err();
        assert!(
            matches!(&error, DecorationLibraryError::Decode { path: actual, .. } if actual == &path)
        );
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .is_some()
        );
        let error = DecorationLibrary::load(&dir).unwrap_err();
        assert!(
            matches!(&error, DecorationLibraryError::Read { path: actual, .. } if actual == &dir)
        );
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .is_some()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
