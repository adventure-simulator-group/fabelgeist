//! Eligibility and selection from the authored starting-profession catalog.
use super::*;

pub(super) fn select_organization(
    profession: StartingProfession,
    seed: &str,
    age_tier: StartingAgeTier,
    slot: u8,
) -> Result<&'static crate::organization::OrganizationDefinition, StartingCharacterError> {
    let eligible = catalog()
        .organizations
        .iter()
        .filter(|organization| {
            organization
                .starting_role
                .as_ref()
                .is_some_and(|role| role.profession == profession)
        })
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return Err(StartingCharacterError::NoEligibleOrganization);
    }
    Ok(eligible[tier_random("organization", seed, age_tier, slot).index(eligible.len())])
}
