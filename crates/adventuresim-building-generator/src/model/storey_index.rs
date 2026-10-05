//! Ordinal into the occupied storeys of a building programme.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreyIndex(usize);
impl StoreyIndex {
    pub const GROUND: Self = Self(0);
    pub const FIRST_UPPER: Self = Self(1);
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    pub const fn index(self) -> usize {
        self.0
    }
}
