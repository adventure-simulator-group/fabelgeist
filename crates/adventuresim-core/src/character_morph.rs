//! Shared identity-morph naming and deterministic cosmetic character variation.

pub const IDENTITY_MORPH_COUNT: usize = 45;
/// One target adds one MHR identity coefficient to the exported recipe.
pub const IDENTITY_MORPH_STEP: f32 = 1.0;
const MAX_IDENTITY_VARIATION: f32 = 0.35;

/// MHR's ordered identity basis: body, head, then hands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityMorph(usize);

impl IdentityMorph {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..IDENTITY_MORPH_COUNT).map(Self)
    }

    pub fn index(self) -> usize {
        self.0
    }

    pub fn name(self) -> String {
        format!("mhr_identity_{:02}", self.0)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let index = name.strip_prefix("mhr_identity_")?.parse().ok()?;
        let target = Self(index);
        (index < IDENTITY_MORPH_COUNT && target.name() == name).then_some(target)
    }
}

/// Cosmetic weights derived from the persistent character ID, independent of
/// entity allocation, spawn order, machine, and item ownership. No combat state.
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterMorphWeights([f32; IDENTITY_MORPH_COUNT]);

impl CharacterMorphWeights {
    pub fn from_character_id(character_id: u64) -> Self {
        let mut random = fabelgeist_determinism::StreamId::new("character.identity-morph")
            .rng(character_id, &[]);
        Self(std::array::from_fn(|_| {
            let unit = random.inclusive_unit_f32();
            (unit * 2.0 - 1.0) * MAX_IDENTITY_VARIATION
        }))
    }

    pub fn named_weight(&self, name: &str) -> Option<f32> {
        IdentityMorph::from_name(name).map(|target| self.0[target.index()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_stable_distinct_bounded_and_named() {
        let first = CharacterMorphWeights::from_character_id(42);
        assert_eq!(first, CharacterMorphWeights::from_character_id(42));
        assert_ne!(first, CharacterMorphWeights::from_character_id(43));
        assert!(first.0.iter().all(|v| v.abs() <= MAX_IDENTITY_VARIATION));
        assert!(first.0.iter().any(|v| *v != 0.0));
        for target in IdentityMorph::all() {
            assert_eq!(
                first.named_weight(&target.name()),
                Some(first.0[target.index()])
            );
        }
        assert_eq!(first.named_weight("mhr_identity_45"), None);
        assert_eq!(first.named_weight("mhr_identity_0"), None);
        assert_eq!(first.named_weight("smile"), None);
    }
}
