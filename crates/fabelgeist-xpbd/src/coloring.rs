//! Greedy constraint coloring preserves arrival order within each color.
//!
//! Constraints sharing a particle occupy different dispatches. Sequential color
//! dispatches retain the solver's Gauss-Seidel order without device races.
use std::collections::HashMap;

use crate::{ConstraintCount, ConstraintIncidence, ConstraintIndex, ParticleIndex};
use fabelgeist_gpu::prelude::{InvocationCount, PassParameters};

/// A color's position in the sequential solve schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintColor(usize);
impl std::fmt::Display for ConstraintColor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorCount(usize);
impl From<usize> for ColorCount {
    fn from(colors: usize) -> Self {
        Self(colors)
    }
}
impl std::fmt::Display for ColorCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A constraint address after permutation into color order.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ConstraintIndex, ConstraintSlot};
/// fn original(_: ConstraintIndex) {}
/// fn wrong(slot: ConstraintSlot) { original(slot); }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintSlot(u32);
impl From<ConstraintSlot> for usize {
    fn from(slot: ConstraintSlot) -> Self {
        slot.0 as usize
    }
}

/// Immutable color interval, in reordered constraint slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorRange {
    color: ConstraintColor,
    first: ConstraintSlot,
    end: ConstraintSlot,
}
impl ColorRange {
    pub fn color(self) -> ConstraintColor {
        self.color
    }
    pub fn count(self) -> ConstraintCount {
        ((self.end.0 - self.first.0) as usize).into()
    }
    pub fn slots(self) -> impl Iterator<Item = ConstraintSlot> {
        (self.first.0..self.end.0).map(ConstraintSlot)
    }
    pub(crate) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("first".into(), self.first.0.into());
        parameters.insert("count".into(), self.count().uniform());
    }
    pub(crate) fn dispatch_items(self) -> InvocationCount {
        self.count().dispatch_items()
    }
}

/// Constraints grouped into colors, with their immutable permutation.
#[derive(Clone, Debug, Default)]
pub struct Coloring {
    order: Vec<ConstraintIndex>,
    ranges: Vec<ConstraintSlot>,
}
impl Coloring {
    pub fn color_count(&self) -> ColorCount {
        self.ranges.len().saturating_sub(1).into()
    }
    pub fn constraint_count(&self) -> ConstraintCount {
        self.order.len().into()
    }
    pub fn order(&self) -> &[ConstraintIndex] {
        &self.order
    }
    pub fn colors(&self) -> ConstraintColors<'_> {
        ConstraintColors {
            ranges: &self.ranges,
            next: ConstraintColor(0),
        }
    }
    pub fn for_incidence(incidence: &ConstraintIncidence) -> Self {
        let records: Vec<&[ParticleIndex]> = incidence.records().collect();
        Self::for_constraints(&records)
    }
    /// Variable-arity graphs may contain empty records and repeated addresses.
    /// Incidence identifies conflicts; it does not validate particle membership.
    pub fn for_constraints(constraints: &[&[ParticleIndex]]) -> Self {
        if constraints.is_empty() {
            return Self {
                order: Vec::new(),
                ranges: vec![ConstraintSlot(0)],
            };
        }
        let mut taken: HashMap<ParticleIndex, Vec<ColorUse>> = HashMap::new();
        let mut colors = Vec::with_capacity(constraints.len());
        let mut color_count = ColorCount(0);
        for particles in constraints {
            let mut candidate = ConstraintColor(0);
            loop {
                let mut free = true;
                for particle in *particles {
                    if let Some(used) = taken.get(particle)
                        && used.get(candidate.0) == Some(&ColorUse::Taken)
                    {
                        free = false;
                        break;
                    }
                }
                if free {
                    break;
                }
                candidate.0 += 1;
            }
            for &particle in *particles {
                let used = taken.entry(particle).or_default();
                if used.len() <= candidate.0 {
                    used.resize(candidate.0 + 1, ColorUse::Free);
                }
                used[candidate.0] = ColorUse::Taken;
            }
            // Preserve the original native u32 color word narrowing.
            colors.push(ConstraintColor(candidate.0 as u32 as usize));
            color_count.0 = color_count.0.max(candidate.0 + 1);
        }
        let mut counts = vec![ConstraintSlot(0); color_count.0 + 1];
        for color in &colors {
            counts[color.0 + 1].0 += 1;
        }
        for i in 1..counts.len() {
            counts[i].0 += counts[i - 1].0;
        }
        let ranges = counts.clone();
        let mut cursor = counts;
        let mut order = vec![ConstraintIndex::from(0); constraints.len()];
        for (index, color) in colors.iter().enumerate() {
            let slot = &mut cursor[color.0];
            order[slot.0 as usize] = ConstraintIndex::from(index as u32);
            slot.0 += 1;
        }
        Self { order, ranges }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorUse {
    Free,
    Taken,
}

pub struct ConstraintColors<'a> {
    ranges: &'a [ConstraintSlot],
    next: ConstraintColor,
}
impl Iterator for ConstraintColors<'_> {
    type Item = ColorRange;
    fn next(&mut self) -> Option<ColorRange> {
        if self.next.0 + 1 >= self.ranges.len() {
            return None;
        }
        let range = ColorRange {
            color: self.next,
            first: self.ranges[self.next.0],
            end: self.ranges[self.next.0 + 1],
        };
        self.next.0 += 1;
        Some(range)
    }
}

#[cfg(test)]
mod tests;
