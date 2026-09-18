//! Historically sourced names for starting-character projections.

use super::{DEFAULT_CHARACTER_AGE_YEARS, StartingCharacterSpec, StartingProfession, StartingSex};
use adventuresim_world_schema::person_names::{
    NameCulture, NameEducation, NameGenerationContext, NameRegister, NameSex, NameSocialClass,
    PersonalNameIdentity, generate_personal_name, render_personal_name,
};

const DEFAULT_PERSONAL_NAME_SEED: u64 = 0xd3fa_1544_0000_0001;

fn historical_name(
    sex: StartingSex,
    age_years: u16,
    stable_seed: u64,
    profession: Option<StartingProfession>,
) -> (PersonalNameIdentity, String) {
    let name_sex = match sex {
        StartingSex::Female => NameSex::Female,
        StartingSex::Male => NameSex::Male,
    };
    let mut context = NameGenerationContext::german_lutheran(
        name_sex,
        crate::strategic_time::WORLD_START_YEAR.saturating_sub(i32::from(age_years)),
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
    let identity = generate_personal_name(context, stable_seed, None)
        .expect("the build-validated German repertoire covers the MVP period");
    let display = render_personal_name(
        &identity,
        NameCulture::German,
        NameRegister::Everyday,
        name_sex,
    )
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
    PersonalNameIdentity::authored(String::new(), NameCulture::German)
}

pub(super) fn default_identity_and_name() -> (PersonalNameIdentity, String) {
    historical_name(
        StartingSex::Male,
        DEFAULT_CHARACTER_AGE_YEARS,
        DEFAULT_PERSONAL_NAME_SEED,
        None,
    )
}

pub fn default_character_name() -> String {
    default_identity_and_name().1
}
