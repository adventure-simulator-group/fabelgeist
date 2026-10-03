//! Historically sourced names for starting-character projections.

use super::{DEFAULT_CHARACTER_AGE_YEARS, StartingCharacterSpec, StartingProfession};
use adventuresim_world_schema::person_names::{
    NameCatalogError, NameEducation, NameGenerationContext, NameRegister, NameSocialClass,
    NameStableSeed, PersonalNameIdentity, RenderedPersonalName, generate_personal_name,
    render_personal_name,
};
use adventuresim_world_schema::{Culture, Sex, calendar::StrategicMinute};

const DEFAULT_PERSONAL_NAME_SEED: u64 = 0xd3fa_1544_0000_0001;

fn historical_name(
    sex: Sex,
    age_years: u16,
    stable_seed: u64,
    profession: Option<StartingProfession>,
) -> PersonalNameIdentity {
    let mut context = NameGenerationContext::german_lutheran(
        sex,
        StrategicMinute::ZERO
            .birth_year_for_age(age_years)
            .expect("starting character age fits the calendar"),
    );
    if profession == Some(StartingProfession::LearnedReligiousPractitioner) {
        context.social_class = NameSocialClass::Clergy;
        context.education = NameEducation::Scholarly;
    } else if matches!(
        profession,
        Some(StartingProfession::Merchant | StartingProfession::WitchHunter)
    ) {
        context.education = NameEducation::Literate;
    }
    generate_personal_name(context, NameStableSeed::new(stable_seed), None)
        .expect("the build-validated German repertoire covers the MVP period")
}

pub(super) fn assign(spec: &mut StartingCharacterSpec, stable_seed: u64) {
    spec.name_identity = historical_name(
        spec.personality.sex,
        spec.age_years,
        stable_seed,
        spec.profession,
    );
}

pub(super) fn pending_identity() -> PersonalNameIdentity {
    PersonalNameIdentity::authored(String::new(), Culture::German)
}

pub(super) fn default_identity() -> PersonalNameIdentity {
    historical_name(
        Sex::Male,
        DEFAULT_CHARACTER_AGE_YEARS,
        DEFAULT_PERSONAL_NAME_SEED,
        None,
    )
}

pub fn default_character_name() -> String {
    let identity = default_identity();
    render_personal_name(
        &identity,
        identity.native_culture,
        NameRegister::Everyday,
        Sex::Male,
    )
    .expect("the canonical default identity has a native everyday rendering")
    .into_string()
}

impl StartingCharacterSpec {
    /// Project the semantic identity at a display or persistence boundary.
    pub fn native_everyday_name(&self) -> Result<RenderedPersonalName, NameCatalogError> {
        render_personal_name(
            &self.name_identity,
            self.name_identity.native_culture,
            NameRegister::Everyday,
            self.personality.sex,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::person_names::{
        GivenNameResolution, NameCatalogError, NameFamilyId, NameFormSelectionSeed,
    };

    #[test]
    fn deterministic_candidates_serialize_only_the_semantic_name() {
        let first = super::super::default_character("owner");
        let second = super::super::default_character("owner");
        assert_eq!(first.native_everyday_name(), second.native_everyday_name());
        let json = serde_json::to_value(&first).unwrap();
        assert!(json.get("name").is_none());
        let restored: StartingCharacterSpec = serde_json::from_value(json).unwrap();
        assert_eq!(
            restored.native_everyday_name(),
            first.native_everyday_name()
        );
    }

    #[test]
    fn changing_identity_changes_the_consumed_projection() {
        let mut spec = super::super::default_character("owner");
        spec.name_identity = PersonalNameIdentity::authored("Authored Person", Culture::German);
        assert_eq!(
            spec.native_everyday_name().unwrap().as_str(),
            "Authored Person"
        );
        spec.name_identity = PersonalNameIdentity {
            given: GivenNameResolution::UnresolvedHistorical {
                possible_family_ids: vec![
                    NameFamilyId::new("johannes"),
                    NameFamilyId::new("heinrich"),
                ],
                recorded_form: "Historical Form".into(),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(0),
            surname_id: None,
        };
        assert_eq!(
            spec.native_everyday_name().unwrap().as_str(),
            "Historical Form"
        );
    }

    #[test]
    fn malformed_identity_is_a_rendering_failure() {
        let mut spec = super::super::default_character("owner");
        spec.name_identity = PersonalNameIdentity::authored("\n", Culture::German);
        assert_eq!(
            spec.native_everyday_name(),
            Err(NameCatalogError::InvalidRenderedName)
        );
    }
}
