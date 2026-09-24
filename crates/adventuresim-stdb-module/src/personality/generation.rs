use super::*;

const BEHAVIOR: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.behavior");
const SEX: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.sex");
const PRESENTATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.presentation");
const INCLINATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.inclination");

pub fn personality_from_stable_seed(character_id: u64, stable_seed: u64) -> CharacterPersonality {
    let sex = if SEX.rng(stable_seed, &[character_id]).boolean() {
        Sex::Female
    } else {
        Sex::Male
    };
    let presentation = match (
        sex,
        PRESENTATION.rng(stable_seed, &[character_id]).index(100),
    ) {
        (_, 0..=3) => Presentation::Ambiguous,
        (Sex::Female, 4) => Presentation::Man,
        (Sex::Male, 4) => Presentation::Woman,
        (Sex::Female, _) => Presentation::Woman,
        (Sex::Male, _) => Presentation::Man,
    };
    personality_from_stable_seed_with_demographics(character_id, stable_seed, sex, presentation)
}

pub fn personality_from_stable_seed_with_demographics(
    character_id: u64,
    stable_seed: u64,
    sex: Sex,
    presentation: Presentation,
) -> CharacterPersonality {
    let mut behavior = BEHAVIOR.rng(stable_seed, &[character_id]);
    let mut result = random_personality(character_id, &mut behavior);
    result.sex = sex;
    result.presentation = presentation;
    result.inclination = match INCLINATION.rng(stable_seed, &[character_id]).index(100) {
        0 => Inclination::Neither,
        1..=4 => Inclination::Either,
        5..=9 => match sex {
            Sex::Female => Inclination::Women,
            Sex::Male => Inclination::Men,
        },
        _ => match sex {
            Sex::Female => Inclination::Men,
            Sex::Male => Inclination::Women,
        },
    };
    result
}
