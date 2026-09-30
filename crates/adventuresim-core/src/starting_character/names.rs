//! Historically sourced names for starting-character projections.

use super::{DEFAULT_CHARACTER_AGE_YEARS, StartingCharacterSpec, StartingProfession};
use adventuresim_world_schema::person_names::{
    NameEducation, NameGenerationContext, NameRegister, NameSocialClass, NameStableSeed,
    PersonalNameIdentity, generate_personal_name, render_personal_name,
};
use adventuresim_world_schema::{Culture, Sex, calendar::StrategicMinute};

const DEFAULT_PERSONAL_NAME_SEED: u64 = 0xd3fa_1544_0000_0001;

fn historical_name(
    sex: Sex,
    age_years: u16,
    stable_seed: u64,
    profession: Option<StartingProfession>,
) -> (PersonalNameIdentity, String) {
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
    let identity = generate_personal_name(context, NameStableSeed::new(stable_seed), None)
        .expect("the build-validated German repertoire covers the MVP period");
    let display = render_personal_name(&identity, Culture::German, NameRegister::Everyday, sex)
        .expect("a generated German identity has a native everyday rendering");
    (identity, display.into_string())
}

pub(super) fn assign(spec: &mut StartingCharacterSpec, stable_seed: u64) {
    let (identity, display) = historical_name(
        spec.personality.sex,
        spec.age_years,
        stable_seed,
        spec.profession,
    );
    spec.name_identity = identity;
    spec.name = display;
}

pub(super) fn pending_identity() -> PersonalNameIdentity {
    PersonalNameIdentity::authored(String::new(), Culture::German)
}

pub(super) fn default_identity_and_name() -> (PersonalNameIdentity, String) {
    historical_name(
        Sex::Male,
        DEFAULT_CHARACTER_AGE_YEARS,
        DEFAULT_PERSONAL_NAME_SEED,
        None,
    )
}

pub fn default_character_name() -> String {
    default_identity_and_name().1
}
