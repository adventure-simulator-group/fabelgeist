//! Botanical identities are distinct from collection locations and GPU words.

/// Catalog identity of a woody species.
///
/// ```compile_fail
/// use fabelgeist_botany::identity::{SpeciesId, PlantIndex};
/// let species: SpeciesId = PlantIndex::from_word(0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpeciesId(u16);

/// Catalog identity of a climbing species.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VineSpeciesId(u16);

/// Stable identity of a woody plant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlantId(u64);

/// Stable identity of a vine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VineId(u64);

/// Stable identity of one module within its plant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u32);

/// Identity of a growth prototype.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrototypeId(u16);

/// Current index in a plant module collection; shedding may change it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleIndex(u32);

/// Start of a module in its plant node collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeOffset(u32);

/// Current index of a woody plant in its ecosystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlantIndex(usize);

/// Current index of a vine in its ecosystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VineIndex(usize);

impl SpeciesId {
    pub const fn from_word(value: u16) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u16 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl VineSpeciesId {
    pub const fn from_word(value: u16) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u16 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl PlantId {
    pub const fn from_word(value: u64) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u64 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl VineId {
    pub const fn from_word(value: u64) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u64 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl ModuleId {
    pub const fn from_word(value: u32) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u32 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl PrototypeId {
    pub const fn from_word(value: u16) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u16 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl ModuleIndex {
    pub const fn from_word(value: u32) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u32 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl NodeOffset {
    pub const fn from_word(value: u32) -> Self {
        Self(value)
    }
    pub const fn word(self) -> u32 {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl PlantIndex {
    pub const fn from_word(value: usize) -> Self {
        Self(value)
    }
    pub const fn word(self) -> usize {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0
    }
}

impl VineIndex {
    pub const fn from_word(value: usize) -> Self {
        Self(value)
    }
    pub const fn word(self) -> usize {
        self.0
    }
    pub const fn index(self) -> usize {
        self.0
    }
}
