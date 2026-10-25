//! Named overview of notable character capabilities.
use super::*;

pub(super) fn character_summary_rail(
    capability: Option<&CharacterCapability>,
    attributes: Option<&CharacterAttributes>,
    skills: Option<&CharacterSkills>,
    combat_profile: CombatTrainingProfile,
    religion_context: Option<OfficialReligion>,
) -> Markup {
    let icons = character_summary_icons(
        capability,
        attributes,
        skills,
        combat_profile,
        religion_context,
    );
    html! {
        (sidebar_section("Summary", html! {
            @if icons.is_empty() {
                p class="text-muted small-copy" { "No notable capabilities." }
            } @else {
                div class="character-summary-icons" role="list"
                    aria-label="Character capability summary" {
                    @for icon in icons {
                        span role="listitem" {
                            span class=(format!(
                                    "character-summary-icon skill-rank-tier-{}",
                                    skill_rank_tier(icon.rank)
                                ))
                                role="button" aria-pressed="false" tabindex="0" aria-label=(&icon.label)
                                data-tooltip-pinnable data-strategic-tooltip=(&icon.tooltip) {
                                span class="summary-readable-label" aria-hidden="true" { (&icon.label) }
                                @match icon.kind {
                                    SummaryIconKind::Mask(path) => {
                                        span class="character-summary-icon-mask"
                                            style=(format!("--summary-icon: url('{path}')"))
                                            aria-hidden="true" {}
                                    }
                                    SummaryIconKind::Monogram { text, germanic_style, written } => {
                                        span class=(format!(
                                                "character-summary-monogram language-{}{}",
                                                if written { "written" } else { "oral" },
                                                if germanic_style { " language-blackletter" } else { "" },
                                            ))
                                            aria-hidden="true" { (text) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
    }
}
