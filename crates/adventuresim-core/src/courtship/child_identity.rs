/// Deterministic child identity inputs shared by conception and birth.
use super::stable_lifecycle_hash;
use crate::personality::Sex;

macro_rules! child_value {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

child_value!(ChildIdentitySeed);
child_value!(ChildNameSeed);
child_value!(HouseholdPlacementSeed);
child_value!(ChildBirthMinute);
child_value!(PregnancyOrdinal);

impl ChildIdentitySeed {
    pub const fn wrapping_add(self, value: u64) -> Self {
        Self(self.0.wrapping_add(value))
    }
}

const CHILD_IDENTITY_DOMAIN: &str = "child-identity";
const CHILD_NAME_DOMAIN: &str = "child-name-v2";
const CHILD_SEX_DOMAIN: &str = "child-sex";
const CHILD_HOME_DOMAIN: &str = "child-home";
const CHILD_SEX_STREAM: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("lifecycle.child-sex");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildSeeds {
    pub identity: ChildIdentitySeed,
    pub name: ChildNameSeed,
    pub sex: Sex,
    pub home: HouseholdPlacementSeed,
}

/// Domain-separated child seeds make identity, naming, sex, and home placement
/// stable without coupling any result to table insertion order.
pub fn deterministic_child_seeds(
    first_parent_id: &str,
    second_parent_id: &str,
    pregnancy_ordinal: PregnancyOrdinal,
    birth_minute: ChildBirthMinute,
    home_location_id: &str,
) -> ChildSeeds {
    let (left, right) = if first_parent_id <= second_parent_id {
        (first_parent_id, second_parent_id)
    } else {
        (second_parent_id, first_parent_id)
    };
    let pregnancy = pregnancy_ordinal.get().to_string();
    let birth = birth_minute.get().to_string();
    let base = [left, right, &pregnancy, &birth];
    ChildSeeds {
        identity: ChildIdentitySeed::new(stable_lifecycle_hash(CHILD_IDENTITY_DOMAIN, &base)),
        name: ChildNameSeed::new(stable_lifecycle_hash(CHILD_NAME_DOMAIN, &base)),
        sex: if CHILD_SEX_STREAM
            .rng(stable_lifecycle_hash(CHILD_SEX_DOMAIN, &base), &[])
            .boolean()
        {
            Sex::Female
        } else {
            Sex::Male
        },
        home: HouseholdPlacementSeed::new(stable_lifecycle_hash(
            CHILD_HOME_DOMAIN,
            &[left, right, &pregnancy, &birth, home_location_id],
        )),
    }
}
