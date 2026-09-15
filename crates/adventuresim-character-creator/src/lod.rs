//! Supported character and equipment export detail levels.

pub use fabelgeist_mhr::{MAX_LOD as MAX_CHARACTER_LOD, MIN_LOD as MIN_CHARACTER_LOD};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterLod(u8);

impl TryFrom<u8> for CharacterLod {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        anyhow::ensure!(
            (MIN_CHARACTER_LOD..=MAX_CHARACTER_LOD).contains(&value),
            "character LOD must be 4..=6; higher resolution is reserved for armor bake sources"
        );
        Ok(Self(value))
    }
}

impl CharacterLod {
    pub fn vertices(self) -> usize {
        [2_461, 971, 595][usize::from(self.0 - MIN_CHARACTER_LOD)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_export_detail() {
        for value in 0..=u8::MAX {
            assert_eq!(
                CharacterLod::try_from(value).is_ok(),
                (4..=6).contains(&value)
            );
        }
    }
}
