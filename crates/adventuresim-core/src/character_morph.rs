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
        // SplitMix64 fixes the visual seed across clients without relying on
        // platform hashing or a random-number library's changing algorithms.
        let mut state = character_id;
        Self(std::array::from_fn(|_| {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut bits = state;
            bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
            bits ^= bits >> 31;
            let unit = (bits >> 40) as f32 / ((1_u32 << 24) - 1) as f32;
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
