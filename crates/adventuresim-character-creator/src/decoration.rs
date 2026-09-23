//! Decoration of plate steel: the engraving cut into a piece and the trim
//! along its edges, and the named library the studio saves them to.

use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{engraving::Engraving, trim::Trim};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, path::Path};

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

    pub fn validate(&self) -> Result<(), String> {
        self.engraving
            .as_ref()
            .map_or(Ok(()), Engraving::validate)?;
        self.trim.as_ref().map_or(Ok(()), Trim::validate)
    }
}

/// The name a decoration is saved under: non-empty, without surrounding space.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DecorationName(String);

impl TryFrom<String> for DecorationName {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("a decoration needs a name".into());
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<DecorationName> for String {
    fn from(name: DecorationName) -> Self {
        name.0
    }
}

impl fmt::Display for DecorationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Named decorations, designed in the studio's armory and chosen for
/// inventory articles. Choosing one copies it into the article, so a recipe
/// never depends on the library.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DecorationLibrary {
    entries: BTreeMap<DecorationName, Decoration>,
}

impl DecorationLibrary {
    /// The library saved at `path`; a library not yet saved there is empty.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let library: Self = serde_json::from_slice(&std::fs::read(path)?)
            .with_context(|| format!("parsing decoration library {}", path.display()))?;
        library
            .validate()
            .with_context(|| format!("checking decoration library {}", path.display()))?;
        Ok(library)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("saving {}", path.display()))
    }

    fn validate(&self) -> Result<()> {
        for (name, decoration) in &self.entries {
            ensure!(!decoration.is_plain(), "{name} decorates nothing");
            decoration
                .validate()
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("invalid decoration {name}"))?;
        }
        Ok(())
    }

    /// Save `decoration` under `name`, replacing any decoration named so.
    pub fn insert(&mut self, name: DecorationName, decoration: Decoration) -> Result<()> {
        ensure!(!decoration.is_plain(), "{name} decorates nothing");
        decoration.validate().map_err(anyhow::Error::msg)?;
        self.entries.insert(name, decoration);
        Ok(())
    }

    pub fn remove(&mut self, name: &DecorationName) -> Option<Decoration> {
        self.entries.remove(name)
    }

    pub fn get(&self, name: &DecorationName) -> Option<&Decoration> {
        self.entries.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DecorationName, &Decoration)> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The name of a saved decoration identical to `decoration`.
    pub fn name_of(&self, decoration: &Decoration) -> Option<&DecorationName> {
        self.entries
            .iter()
            .find_map(|(name, saved)| (saved == decoration).then_some(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::ornament::Ornament;

    fn name(name: &str) -> DecorationName {
        DecorationName::try_from(name.to_owned()).unwrap()
    }

    fn vine() -> Decoration {
        Decoration {
            engraving: Some(Engraving::ornament(Ornament::default())),
            trim: Some(Trim::default()),
        }
    }

    #[test]
    fn names_are_trimmed_and_never_blank() {
        assert_eq!(name("  Gilded vine ").to_string(), "Gilded vine");
        assert!(DecorationName::try_from("   ".to_owned()).is_err());
        assert!(serde_json::from_str::<DecorationName>("\"\"").is_err());
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
        assert!(library.insert(name("Bare"), Decoration::default()).is_err());
        let mut broken = vine();
        broken.trim.as_mut().unwrap().width = f32::NAN;
        assert!(library.insert(name("Broken"), broken).is_err());
        assert!(library.is_empty());
    }
}
