//! Weighted German-Christian names for the 1544 MVP region.
//!
//! Weights are authored estimates, not measured regional frequencies. Historical
//! sources, spelling choices and limitations are documented below.

#![doc = include_str!("person_names.md")]

mod catalog;

/// Name repertoire shared by residents, household labels and proprietor signs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamePool {
    Female,
    Male,
    Surname,
}

impl NamePool {
    /// Selects a name family by weight, then an equally weighted display form.
    ///
    /// Supply uniformly distributed entropy. All arithmetic stays in `u64`, so
    /// native and wasm32 builds select the same name. Adding a spelling variant
    /// does not increase its family's share of the population.
    pub fn choose(self, entropy: u64) -> &'static str {
        let families = match self {
            Self::Female => catalog::FEMALE,
            Self::Male => catalog::MALE,
            Self::Surname => catalog::SURNAMES,
        };
        NameFamily::choose(families, entropy)
    }
}

/// Relative frequency of a name family in the authored MVP population.
#[derive(Clone, Copy)]
struct NameWeight(u16);

struct NameFamily {
    weight: NameWeight,
    forms: &'static [&'static str],
}

impl NameFamily {
    const fn new(weight: u16, forms: &'static [&'static str]) -> Self {
        assert!(weight > 0 && !forms.is_empty());
        Self {
            weight: NameWeight(weight),
            forms,
        }
    }

    fn choose(families: &[Self], entropy: u64) -> &'static str {
        let total: u64 = families
            .iter()
            .map(|family| u64::from(family.weight.0))
            .sum();
        let mut ticket = entropy % total;
        for family in families {
            let weight = u64::from(family.weight.0);
            if ticket < weight {
                let variant = (entropy / total) % family.forms.len() as u64;
                return family.forms[variant as usize];
            }
            ticket -= weight;
        }
        unreachable!("positive family weights cover every ticket")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn family_weights_survive_spelling_variants_and_repeat_at_full_cycles() {
        let families = [
            NameFamily::new(3, &["Hans", "Johann"]),
            NameFamily::new(1, &["Peter"]),
        ];
        let names: Vec<_> = (0..8)
            .map(|entropy| NameFamily::choose(&families, entropy))
            .collect();
        assert_eq!(
            names,
            [
                "Hans", "Hans", "Hans", "Peter", "Johann", "Johann", "Johann", "Peter"
            ]
        );
        assert_eq!(NameFamily::choose(&families, 8), "Hans");
        assert_eq!(NameFamily::choose(&families, u64::MAX), "Peter");
    }

    #[test]
    fn selection_preserves_entropy_above_the_wasm_pointer_range() {
        let families = [
            NameFamily::new(3, &["Hans", "Johann"]),
            NameFamily::new(2, &["Peter"]),
        ];
        // This period does not divide 2^32: truncation would incorrectly return Hans.
        assert_eq!(
            NameFamily::choose(&families, u64::from(u32::MAX) + 1),
            "Johann"
        );
    }

    #[test]
    fn catalogs_have_positive_weights_and_unique_nonempty_display_forms() {
        for families in [catalog::FEMALE, catalog::MALE, catalog::SURNAMES] {
            let mut names = BTreeSet::new();
            for family in families {
                assert!(family.weight.0 > 0);
                assert!(!family.forms.is_empty());
                for form in family.forms {
                    assert!(!form.trim().is_empty());
                    assert!(names.insert(*form), "duplicate display form: {form}");
                }
            }
        }
    }
}
