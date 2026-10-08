// Handwritten facade over the generated SpacetimeDB client bindings.
// Regenerate `mod.rs` and its sibling binding files with: just generate-db-client

#[path = "src/mod.rs"]
mod bindings;

pub use bindings::*;
pub use spacetimedb_sdk;

// SDK records mirror the schema. This is the type-to-type projection into the
// shared owner used by domain logic, with the same complete root word.
impl From<fabelgeist_determinism::Seed> for Seed {
    fn from(seed: fabelgeist_determinism::Seed) -> Self {
        Self {
            word: seed.to_u64(),
        }
    }
}
impl From<Seed> for fabelgeist_determinism::Seed {
    fn from(seed: Seed) -> Self {
        Self::from_u64(seed.word)
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fabelgeist_determinism::Seed::from(self.clone()).fmt(formatter)
    }
}
