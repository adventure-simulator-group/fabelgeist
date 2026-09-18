//! Child focus and age-appropriate training allocation.
use super::*;

pub(super) fn deterministic_child_focus(seed: u64) -> ChildActivityFocus {
    let choice = fabelgeist_determinism::StreamId::new("character.child-focus")
        .rng(seed, &[])
        .index(4);
    match choice {
        0 => ChildActivityFocus::Play,
        1 => ChildActivityFocus::Study,
        2 => ChildActivityFocus::HouseholdHelp,
        _ => ChildActivityFocus::SocialLearning,
    }
}

pub(super) fn focus_training(focus: ChildActivityFocus, adolescent: bool) -> [(Skill, f32); 2] {
    let daily_hours = if adolescent { 3.0 } else { 2.0 };
    match focus {
        ChildActivityFocus::Play => [
            (Skill::Balance, daily_hours * 0.55),
            (Skill::Dodge, daily_hours * 0.45),
        ],
        ChildActivityFocus::Study => [
            (Skill::Insight, daily_hours * 0.6),
            (Skill::Physiology, daily_hours * 0.4),
        ],
        ChildActivityFocus::HouseholdHelp => [
            (Skill::Cooking, daily_hours * 0.6),
            (Skill::Tailoring, daily_hours * 0.4),
        ],
        ChildActivityFocus::SocialLearning => [
            (Skill::Charm, daily_hours * 0.55),
            (Skill::Insight, daily_hours * 0.45),
        ],
    }
}

pub(super) fn curriculum_real_hours(
    focus: ChildActivityFocus,
    track: usize,
    birth_minute: u64,
    start_minute: u64,
    end_minute: u64,
) -> (Skill, f32) {
    let six = birth_minute.saturating_add(6 * MINUTES_PER_YEAR);
    let twelve = birth_minute.saturating_add(12 * MINUTES_PER_YEAR);
    let sixteen = birth_minute.saturating_add(u64::from(ADULT_AGE_YEARS) * MINUTES_PER_YEAR);
    let middle_minutes = end_minute
        .min(twelve)
        .saturating_sub(start_minute.max(six).min(end_minute.min(twelve)));
    let adolescent_minutes = end_minute
        .min(sixteen)
        .saturating_sub(start_minute.max(twelve).min(end_minute.min(sixteen)));
    let middle = focus_training(focus, false)[track];
    let adolescent = focus_training(focus, true)[track];
    debug_assert_eq!(middle.0, adolescent.0);
    let hours = middle_minutes as f32 / MINUTES_PER_DAY as f32 * middle.1
        + adolescent_minutes as f32 / MINUTES_PER_DAY as f32 * adolescent.1;
    (middle.0, hours)
}
