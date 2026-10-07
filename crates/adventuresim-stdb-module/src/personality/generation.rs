use super::*;
use crate::character::CharacterId;
use fabelgeist_determinism::Seed;

/// Stable entropy purpose for persisted character personality generation.
pub(super) const PERSONALITY_GENERATION_DOMAIN: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality");

const BEHAVIOR: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.behavior");
const SEX: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.sex");
const PRESENTATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.presentation");
const INCLINATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character.personality.inclination");

pub(crate) fn personality_from_stable_seed(
    character_id: CharacterId,
    stable_seed: Seed,
) -> CharacterPersonality {
    let sex = *SEX
        .rng(stable_seed, &[character_id.get()])
        .choose(Sex::VARIANTS);
    let presentation = match (
        sex,
        PRESENTATION
            .rng(stable_seed, &[character_id.get()])
            .index(100),
    ) {
        (_, 0..=3) => Presentation::Ambiguous,
        (Sex::Female, 4) => Presentation::Man,
        (Sex::Male, 4) => Presentation::Woman,
        (Sex::Female, _) => Presentation::Woman,
        (Sex::Male, _) => Presentation::Man,
    };
    personality_from_stable_seed_with_demographics(character_id, stable_seed, sex, presentation)
}

pub(crate) fn personality_from_stable_seed_with_demographics(
    character_id: CharacterId,
    stable_seed: Seed,
    sex: Sex,
    presentation: Presentation,
) -> CharacterPersonality {
    let mut behavior = BEHAVIOR.rng(stable_seed, &[character_id.get()]);
    let mut result = random_personality(character_id, &mut behavior);
    result.sex = sex;
    result.presentation = presentation;
    result.inclination = match INCLINATION
        .rng(stable_seed, &[character_id.get()])
        .index(100)
    {
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

/// Generate a sparse profile with exactly two through four distinct axes.
pub(crate) fn random_personality(
    character_id: CharacterId,
    random: &mut DeterministicRng,
) -> CharacterPersonality {
    let mut result = CharacterPersonality::neutral(character_id.get());
    let mut axes = [0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    random.shuffle(&mut axes);
    let count = 2 + random.index(3);
    for axis in axes.into_iter().take(count) {
        apply_axis(&mut result, axis, random);
    }
    assign_demographics(&mut result, random);
    result
}

fn assign_demographics(result: &mut CharacterPersonality, random: &mut DeterministicRng) {
    result.sex = *random.choose(Sex::VARIANTS);
    let presentation_roll = random.index(100);
    result.presentation = match (result.sex, presentation_roll) {
        (_, 0..=3) => Presentation::Ambiguous,
        (Sex::Female, 4) => Presentation::Man,
        (Sex::Male, 4) => Presentation::Woman,
        (Sex::Female, _) => Presentation::Woman,
        (Sex::Male, _) => Presentation::Man,
    };
    // Direction is generated from demographic sex rather than the public
    // presentation signal. The signal is the only field attraction consumes.
    result.inclination = match random.index(100) {
        0 => Inclination::Neither,
        1..=4 => Inclination::Either,
        5..=9 => match result.sex {
            Sex::Female => Inclination::Women,
            Sex::Male => Inclination::Men,
        },
        _ => match result.sex {
            Sex::Female => Inclination::Men,
            Sex::Male => Inclination::Women,
        },
    };
}

fn apply_axis(result: &mut CharacterPersonality, axis: u8, random: &mut DeterministicRng) {
    match axis {
        0 => {
            result.nerve = if random.boolean() {
                Nerve::Brave
            } else {
                Nerve::Fearful
            }
        }
        1 => {
            result.drive = if random.boolean() {
                Drive::Ambitious
            } else {
                Drive::Content
            }
        }
        2 => {
            result.outlook = if random.boolean() {
                Outlook::Sanguine
            } else {
                Outlook::Brooding
            }
        }
        3 => {
            result.sociability = if random.boolean() {
                Sociability::Gregarious
            } else {
                Sociability::Solitary
            }
        }
        4 => {
            result.conscience = match random.index(3) {
                0 => Conscience::Compassionate,
                1 => Conscience::Callous,
                _ => Conscience::Cruel,
            }
        }
        5 => {
            result.self_regard = if random.boolean() {
                SelfRegard::Proud
            } else {
                SelfRegard::Humble
            }
        }
        6 => {
            result.conviction = if random.boolean() {
                Conviction::Zealous
            } else {
                Conviction::Irreverent
            }
        }
        7 => {
            result.hygiene = if random.boolean() {
                Hygiene::Slovenly
            } else {
                Hygiene::Cleanly
            }
        }
        8 => {
            result.temperance = if random.boolean() {
                Temperance::Temperate
            } else {
                Temperance::Drunkard
            }
        }
        9 => {
            result.mirth = if random.boolean() {
                Mirth::Merry
            } else {
                Mirth::Grave
            }
        }
        10 => {
            result.courtship = if random.boolean() {
                Courtship::Amorous
            } else {
                Courtship::Proper
            }
        }
        11 => {
            result.transparency = if random.boolean() {
                Transparency::Open
            } else {
                Transparency::Guarded
            }
        }
        _ => {
            result.self_knowledge = if random.boolean() {
                SelfKnowledge::Introspective
            } else {
                SelfKnowledge::SelfDeceiving
            }
        }
    }
}
