//! Dense values in the ordinal order of one admitted rig.

use crate::RigJointOrdinal;
use derive_more::From;
use std::ops::{Index, IndexMut};

/// A rig-order column. Ordinals select joints rather than arbitrary native slots.
/// Its creator must use the same admitted rig order for every associated column;
/// this type does not prove that two columns belong to the same rig instance.
#[derive(Clone, Debug, PartialEq, From)]
pub struct RigJointTable<T>(Vec<T>);

impl<T> RigJointTable<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.0.iter_mut()
    }

    pub fn indexed(&self) -> impl Iterator<Item = (RigJointOrdinal, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(slot, value)| (slot.into(), value))
    }

    pub fn ordinals(&self) -> impl Iterator<Item = RigJointOrdinal> + use<T> {
        (0..self.len()).map(RigJointOrdinal::from)
    }

    /// Checked lookup for an ordinal whose membership has not been established.
    pub fn get(&self, joint: RigJointOrdinal) -> Option<&T> {
        self.0.get(usize::from(joint))
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.0.fill(value);
    }
}

impl<T> Default for RigJointTable<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> FromIterator<T> for RigJointTable<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self(values.into_iter().collect())
    }
}

impl<T> Index<RigJointOrdinal> for RigJointTable<T> {
    type Output = T;

    /// Direct access is for ordinals established by this rig's construction or
    /// traversal. Unproven external ordinals must use `get` instead.
    fn index(&self, joint: RigJointOrdinal) -> &Self::Output {
        &self.0[usize::from(joint)]
    }
}

impl<T> IndexMut<RigJointOrdinal> for RigJointTable<T> {
    fn index_mut(&mut self, joint: RigJointOrdinal) -> &mut Self::Output {
        &mut self.0[usize::from(joint)]
    }
}

impl<T> IntoIterator for RigJointTable<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a RigJointTable<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RigJointTable<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
