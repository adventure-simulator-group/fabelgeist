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
    Seed(seed)
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

/// Eight little-endian bytes used directly as the SplitMix64 initial state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Seed([u8; 8]);

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
        Self(value.to_le_bytes())
    }

    pub const fn to_u64(self) -> u64 {
        u64::from_le_bytes(self.0)
    }

    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0
    }

    pub fn child(self, stream: StreamId, context: &[&[u8]]) -> Self {
        Self::derive(&self.0, stream, context)
    }

    pub fn rng(self) -> crate::DeterministicRng {
        crate::DeterministicRng::new(self)
    }
}

// Numeric documents preserve the root word; derivation still frames its eight
// little-endian bytes exactly once.
impl serde::Serialize for Seed {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.to_u64())
    }
}
impl<'de> serde::Deserialize<'de> for Seed {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <u64 as serde::Deserialize>::deserialize(deserializer).map(Self::from_u64)
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
