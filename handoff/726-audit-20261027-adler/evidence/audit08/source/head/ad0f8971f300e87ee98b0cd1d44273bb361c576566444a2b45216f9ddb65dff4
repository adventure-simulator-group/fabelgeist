//! Length-framed, domain-separated deterministic seeds.

const DERIVATION_CONTEXT: &str = "fabelgeist.determinism.seed.v1";

fn derivation_hasher(root: &[u8], stream: StreamId) -> blake3::Hasher {
    static BASE: std::sync::OnceLock<blake3::Hasher> = std::sync::OnceLock::new();
    let mut hasher = BASE
        .get_or_init(|| blake3::Hasher::new_derive_key(DERIVATION_CONTEXT))
        .clone();
    for field in [root, stream.0.as_bytes()] {
        hasher.update(&(field.len() as u64).to_le_bytes());
        hasher.update(field);
    }
    hasher
}

fn finish(hasher: blake3::Hasher) -> Seed {
    let mut seed = [0; 8];
    seed.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    Seed::from_u64(u64::from_le_bytes(seed))
}

/// A purpose owned by the caller, independent of other purposes' draw counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamId(&'static str);

impl StreamId {
    pub const fn new(name: &'static str) -> Self {
        assert!(!name.is_empty(), "random streams require a purpose name");
        Self(name)
    }

    /// Derive numeric context using fixed-width little-endian words.
    pub fn seed(self, root: Seed, context: &[u64]) -> Seed {
        let mut hasher = derivation_hasher(&root.to_le_bytes(), self);
        for value in context {
            let field = value.to_le_bytes();
            hasher.update(&(field.len() as u64).to_le_bytes());
            hasher.update(&field);
        }
        finish(hasher)
    }

    pub fn rng(self, root: Seed, context: &[u64]) -> crate::DeterministicRng {
        self.seed(root, context).rng()
    }
}

/// A deterministic root word, framed as eight little-endian bytes for sampling.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb_sats::SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(crate = spacetimedb_sats))]
pub struct Seed {
    word: u64,
}

impl Seed {
    /// Derive from a root, named purpose, and ordered context fields.
    ///
    /// Numeric fields must use fixed-width little-endian encoding. Context
    /// ordinals must denote stable domain slots, never incidental iteration.
    pub fn derive(root: &[u8], stream: StreamId, context: &[&[u8]]) -> Self {
        let mut hasher = derivation_hasher(root, stream);
        for field in context {
            hasher.update(&(field.len() as u64).to_le_bytes());
            hasher.update(field);
        }
        finish(hasher)
    }

    /// A root word or an already derived seed at a serialization boundary.
    pub const fn from_u64(value: u64) -> Self {
        Self { word: value }
    }

    pub const fn to_u64(self) -> u64 {
        self.word
    }

    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.word.to_le_bytes()
    }

    /// Visit an adjacent root word with the existing wrapping seed arithmetic.
    pub const fn wrapping_offset(self, offset: u64) -> Self {
        Self::from_u64(self.to_u64().wrapping_add(offset))
    }

    /// Apply a numeric bitmask at an owned seed arithmetic boundary.
    pub const fn xor_word(self, mask: u64) -> Self {
        Self::from_u64(self.to_u64() ^ mask)
    }

    /// Interpret the existing seed bits as `[0, 1)` without consuming a draw.
    pub fn unit_f32(self) -> f32 {
        crate::unit_f32(self.to_u64())
    }

    /// Interpret the existing seed bits as `[0, 1]` without consuming a draw.
    pub fn inclusive_unit_f32(self) -> f32 {
        crate::inclusive_unit_f32(self.to_u64())
    }

    /// Interpret the existing seed bits as `[0, 1)` without consuming a draw.
    pub fn unit_f64(self) -> f64 {
        crate::unit_f64(self.to_u64())
    }

    pub fn child(self, stream: StreamId, context: &[&[u8]]) -> Self {
        Self::derive(&self.to_le_bytes(), stream, context)
    }

    pub fn rng(self) -> crate::DeterministicRng {
        crate::DeterministicRng::new(self)
    }
}

// Preserve the canonical eight-byte hash input independently of record encoding.
impl std::hash::Hash for Seed {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.to_le_bytes(), state);
    }
}

// Numeric ordering preserves the original word order of seed-keyed lattices.
impl Ord for Seed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_u64().cmp(&other.to_u64())
    }
}
impl PartialOrd for Seed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::fmt::Display for Seed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_u64().fmt(formatter)
    }
}
impl std::fmt::LowerHex for Seed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.to_u64(), formatter)
    }
}
impl std::str::FromStr for Seed {
    type Err = std::num::ParseIntError;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        word.parse().map(Self::from_u64)
    }
}

impl rand::distr::Distribution<Seed> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, random: &mut R) -> Seed {
        Seed::from_u64(random.random())
    }
}

// SpacetimeDB currently supplies its transaction RNG through Rand 0.8.
// Sampling a Seed consumes the same one-word draw as its native u64 output.
#[cfg(feature = "spacetimedb")]
impl rand08::distributions::Distribution<Seed> for rand08::distributions::Standard {
    fn sample<R: rand08::Rng + ?Sized>(&self, random: &mut R) -> Seed {
        Seed::from_u64(random.r#gen())
    }
}

impl From<u64> for Seed {
    fn from(word: u64) -> Self {
        Self::from_u64(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_numeric_seeds_preserve_original_length_framing() {
        for root in [0_u64, 42, u64::MAX] {
            for stream in [
                StreamId::new("grass.community"),
                StreamId::new("building.rooms"),
            ] {
                for context in [vec![], vec![0], vec![1, 2, u64::MAX], vec![42; 32]] {
                    let fields: Vec<_> = context.iter().map(|v: &u64| v.to_le_bytes()).collect();
                    let mut original = blake3::Hasher::new_derive_key(DERIVATION_CONTEXT);
                    for field in [root.to_le_bytes().as_slice(), stream.0.as_bytes()]
                        .into_iter()
                        .chain(fields.iter().map(|v| v.as_slice()))
                    {
                        original.update(&(field.len() as u64).to_le_bytes());
                        original.update(field);
                    }
                    assert_eq!(
                        stream.seed(Seed::from_u64(root), &context),
                        finish(original)
                    );
                    assert_eq!(
                        stream.seed(Seed::from_u64(root), &context),
                        Seed::derive(
                            &root.to_le_bytes(),
                            stream,
                            &fields.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
                        )
                    );
                }
            }
        }
    }
}
