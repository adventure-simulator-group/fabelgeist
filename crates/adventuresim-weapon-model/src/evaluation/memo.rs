//! Bounded memoization of successful pure physical evaluations.
use crate::DerivedProperties;
use std::{collections::VecDeque, sync::Mutex};

// Each recipe is already bounded by the canonical transport/structure limits.
// Keep a fixed number of full design keys, never caller-supplied hash keys.
const RETAINED_EVALUATIONS: usize = 64;

pub(super) struct PhysicalMemo<D>(Mutex<VecDeque<(D, DerivedProperties)>>);
impl<D: Clone + PartialEq> PhysicalMemo<D> {
    pub(super) const fn new() -> Self {
        Self(Mutex::new(VecDeque::new()))
    }
    pub(super) fn evaluate<E>(
        &self,
        design: &D,
        construct: impl FnOnce() -> Result<DerivedProperties, E>,
    ) -> Result<DerivedProperties, E> {
        {
            let entries = self.0.lock().expect("physical memo poisoned");
            if let Some((_, physical)) = entries.iter().find(|(key, _)| key == design) {
                return Ok(*physical);
            }
        }
        // Do not hold the memo lock during construction. Failure leaves no entry.
        let physical = construct()?;
        let mut entries = self.0.lock().expect("physical memo poisoned");
        if !entries.iter().any(|(key, _)| key == design) {
            if entries.len() == RETAINED_EVALUATIONS {
                entries.pop_front();
            }
            entries.push_back((design.clone(), physical));
        }
        Ok(physical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        default_design, derive_properties,
        recipe::{Metres, Shape},
    };
    use std::cell::Cell;

    #[test]
    fn complete_design_keys_reuse_success_and_recompute_edits_and_evictions() {
        let memo = PhysicalMemo::new();
        let calls = Cell::new(0);
        let base = default_design("longsword").unwrap();
        let evaluate = |design: &crate::WeaponDesign| {
            memo.evaluate(design, || {
                calls.set(calls.get() + 1);
                derive_properties(design)
            })
        };
        let expected = evaluate(&base).unwrap();
        assert_eq!(evaluate(&base).unwrap(), expected);
        assert_eq!(calls.get(), 1);
        let mut edited = base.clone();
        let blade = edited
            .recipe
            .components
            .iter_mut()
            .find_map(|component| {
                if let Shape::LoftedBlade(blade) = &mut component.shape {
                    Some(blade)
                } else {
                    None
                }
            })
            .unwrap();
        blade.length = Metres::new(blade.length.get() + 0.1).unwrap();
        let changed = evaluate(&edited).unwrap();
        assert_ne!(changed, expected);
        assert_eq!(changed, crate::generate(&edited).unwrap().derived);
        assert_eq!(calls.get(), 2);
        for index in 0..RETAINED_EVALUATIONS {
            let mut named = base.clone();
            named.recipe.components[0].label = Some(format!("grip {index}"));
            assert_eq!(evaluate(&named).unwrap(), expected);
        }
        let previous = calls.get();
        assert_eq!(evaluate(&base).unwrap(), expected);
        assert_eq!(calls.get(), previous + 1);
        assert_eq!(memo.0.lock().unwrap().len(), RETAINED_EVALUATIONS);
    }

    #[test]
    fn failed_evaluations_are_not_retained() {
        let memo = PhysicalMemo::new();
        let calls = Cell::new(0);
        for _ in 0..2 {
            assert!(
                memo.evaluate(&(), || {
                    calls.set(calls.get() + 1);
                    Err::<DerivedProperties, _>("invalid construction")
                })
                .is_err()
            );
        }
        assert_eq!(calls.get(), 2);
        assert!(memo.0.lock().unwrap().is_empty());
    }
}
