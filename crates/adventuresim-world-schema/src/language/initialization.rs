//! Initial individual language exposure from settlement demographics.
use super::*;

/// Initialize direct oral hours from the character's final settlement.
/// Yiddish is an individual deterministic incidence, never a settlement-wide replacement.
pub fn initial_oral_languages(
    profile: SettlementLanguageProfile,
    character_id: u64,
    npc: bool,
) -> OralLanguageHours {
    let selected = StreamId::new("character.vernacular")
        .rng(character_id, &[])
        .weighted_index(&[
            u64::from(profile.east_central_bp),
            u64::from(profile.west_central_bp),
            u64::from(profile.low_bp),
        ])
        .expect("normalized language profile has a positive total");
    let german = [
        OralLanguage::EastCentral,
        OralLanguage::WestCentral,
        OralLanguage::Low,
    ][selected];
    let yiddish = npc
        && YIDDISH_INCIDENCE_DOMAIN
            .rng(character_id, &[])
            .index(usize::from(crate::BASIS_POINTS_PER_WHOLE))
            < usize::from(profile.yiddish_incidence_bp);
    let mut hours = OralLanguageHours::default();
    *hours.direct_mut(german) = if yiddish {
        ORAL_FLUENCY_HOURS
            * (YIDDISH_LOCAL_GERMAN_FLUENCY - OralLanguage::Yiddish.correlation(german)).max(0.0)
    } else {
        ORAL_FLUENCY_HOURS
    };
    if yiddish {
        hours.yiddish = ORAL_FLUENCY_HOURS;
    }
    hours
}

pub fn initial_character_languages(
    profile: SettlementLanguageProfile,
    character_id: u64,
    npc: bool,
) -> (OralLanguageHours, WrittenLanguageHours) {
    let oral = initial_oral_languages(profile, character_id, npc);
    // Literacy is social and institutional, never a universal consequence of
    // speaking the local language. Noble-family roles and authored professional
    // curricula are applied by character authority after relational roles are
    // established.
    let written = WrittenLanguageHours::default();
    (oral, written)
}
