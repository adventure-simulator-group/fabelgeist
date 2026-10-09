//! Character selection and creation templates.

use adventuresim_core::equipment::LoadoutSlot;
mod candidate;
use candidate::CandidatePresentation;

use maud::{Markup, html};

use super::settlement::{
    CharacterPortraitView, CharacterSheetActions, CharacterSheetView, character_portrait_overlay,
    character_sheet_markup,
};
use super::{entry_layout, item_display_name, item_type_icon, panel, sidebar_section};
use crate::medical::MedicalPresentation;
use crate::spacetimedb::{BackendDevelopmentScenario, CharacterView};
use adventuresim_core::starting_character::{StartingAgeTier, StartingCharacterSpec};

/// List all characters and select the adventurer who enters the strategic layer.
pub fn characters_list_page(
    characters: &[CharacterView],
    scenarios: &[BackendDevelopmentScenario],
    current_character_id: Option<u64>,
) -> Markup {
    let content = html! {
        aside class="left-sidebar" {
            (sidebar_section("Choose an adventurer", html! {
                p class="small-copy text-muted" { "Continue with an adventurer, or create another." }
            }))
        }

        main class="center-content" {
            h2 class="page-title" { "Select your adventurer" }
            @if characters.is_empty() {
                div class="center-welcome" {
                    p { "No adventurers have been created in this browser yet." }
                }
            } @else {
                div class="character-select-grid" {
                    @for character in characters {
                        @let is_current = current_character_id == Some(character.id);
                        (panel(&character.name, html! {
                            div class="stat-grid" {
                                div class="stat-item" {
                                    span class="stat-label" { "Status" }
                                    span class="stat-value" {
                                        @if character.alive { "Alive" } @else { span class="badge badge-danger" { "Dead" } }
                                    }
                                }
                            }
                            @if is_current {
                                p class="text-accent small-copy" {
                                    @if character.alive { "Currently selected" } @else { "Currently viewed" }
                                }
                            }
                            form action=(format!("/characters/{}/select", character.id)) method="post" class="mt-1" {
                                button type="submit" class="btn btn-primary btn-block character-select-action" {
                                    @if !character.alive { "View " (&character.name) }
                                    @else if is_current { "Continue" }
                                    @else { "Play as " (&character.name) }
                                }
                            }
                        }))
                    }
                }
            }
            a href="/characters/candidates" class="btn btn-secondary candidate-play-action" {
                "Create another adventurer"
            }
            @if !scenarios.is_empty() {
                section class="panel mt-2" data-development-scenarios {
                    h2 { "Test scenarios" }
                    p class="small-copy text-muted" { "Search and enter a prebuilt strategic state. Reset the isolated profile to restore every scenario." }
                    label for="scenario-search" class="sr-only" { "Search test scenarios" }
                    input id="scenario-search" type="search" placeholder="Search scenarios" data-scenario-search;
                    @for category in scenarios.iter().map(|scenario| &scenario.category).collect::<std::collections::BTreeSet<_>>() {
                        section data-scenario-group {
                            h3 { (category) }
                            div class="character-select-grid" {
                                @for scenario in scenarios.iter().filter(|scenario| &scenario.category == category) {
                                    article class="panel" data-scenario-card data-scenario-search-text=(format!("{} {} {}", scenario.category, scenario.label, scenario.description).to_ascii_lowercase()) {
                                        h4 { (&scenario.label) }
                                        p class="small-copy" { (&scenario.description) }
                                        form action=(format!("/characters/{}/select", scenario.primary_character_id)) method="post" {
                                            input type="hidden" name="next" value=(&scenario.entry_route);
                                            button type="submit" class="btn btn-primary btn-block" { "Select and open" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                script src="/static/development-scenarios.js?v=1" defer {}
            }
        }

        aside class="right-sidebar" {
            (sidebar_section("Starting settlement", html! {
                p class="small-copy text-muted" { "New adventurers begin at a random settlement with basic supplies." }
            }))
        }
    };

    entry_layout("Select Adventurer", content)
}

pub fn character_switcher_options(
    characters: &[CharacterView],
    current_character_id: Option<u64>,
) -> Markup {
    html! {
        div class="character-switcher-options" {
            @if characters.is_empty() {
                p class="character-switcher-empty" { "No remembered adventurers." }
            } @else {
                @for character in characters {
                    @let current = current_character_id == Some(character.id);
                    form action=(format!("/characters/{}/select", character.id))
                        method="post" data-hard-navigation {
                        button type="submit"
                            class=(if current { "character-switcher-option is-current" } else { "character-switcher-option" })
                            data-character-id=(character.id)
                            aria-current=(if current { "true" } else { "false" }) {
                            span class="character-switcher-option-portrait" aria-hidden="true" {
                                span data-bevy-character=(character.id) { (character.name.chars().next().unwrap_or('?')) }
                            }
                            span class="character-switcher-option-copy" {
                                strong { (&character.name) }
                                small {
                                    @if current { "Currently playing" }
                                    @else if character.alive { "Play this character" }
                                    @else { "View this character" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{character_switcher_options, characters_list_page};
    use crate::spacetimedb::CharacterView;

    #[test]
    fn dead_character_is_labeled_and_uses_view_wording() {
        let character = CharacterView {
            id: 7,
            name: "Fallen Adventurer".into(),
            xp: 0,
            level: 1,
            current_settlement_id: Some("ironforge".into()),
            current_case_site_id: None,
            party_id: Some("solo-7".into()),
            age_years: 30,
            alive: false,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        };

        let markup = characters_list_page(&[character], &[], Some(7)).into_string();
        assert!(markup.contains("Dead"));
        assert!(markup.contains("Currently viewed"));
        assert!(markup.contains("View Fallen Adventurer"));
        assert!(!markup.contains("Play as Fallen Adventurer"));
        assert!(!markup.contains(">Continue<"));
    }

    #[test]
    fn switcher_lists_remembered_characters_before_creation_link() {
        let character = |id, name: &str| CharacterView {
            id,
            name: name.into(),
            xp: 0,
            level: 1,
            current_settlement_id: Some("riverdale".into()),
            current_case_site_id: None,
            party_id: Some(format!("solo-{id}")),
            age_years: 22,
            alive: true,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        };
        let markup =
            character_switcher_options(&[character(7, "Ada"), character(9, "Beatrix")], Some(9))
                .into_string();
        assert!(markup.contains("Ada"));
        assert!(markup.contains("Beatrix"));
        assert!(markup.contains("action=\"/characters/7/select\""));
        assert!(markup.contains("data-hard-navigation"));
        assert_eq!(markup.matches("aria-current=\"true\"").count(), 1);
        assert!(markup.contains("Currently playing"));
    }
}

pub fn character_candidates_bootstrap_page(version: u16) -> Markup {
    let content = html! {
        main class="center-content candidate-bootstrap" {
            h2 class="page-title" { "Choose a stage of life" }
            p { "Choose a starting background, then compare the adventurers available. Your choice is final only when you select Play." }
            noscript { p role="alert" { "JavaScript is required to prepare a private candidate roster." } }
            div data-candidate-bootstrap data-generator-version=(version) {}
            nav class="candidate-age-options" aria-label="Starting age" {
                a class="candidate-age-option" data-candidate-age="young" href="#" {
                    strong { "Young" } span { "Age 16 — No profession" } p { "Start without a trade and find your vocation." }
                }
                a class="candidate-age-option" data-candidate-age="adult" href="#" {
                    strong { "Adult" } span { "Age 22 — Newly qualified" } p { "Begin with a profession at journeyman level." }
                }
                a class="candidate-age-option" data-candidate-age="old" href="#" {
                    strong { "Experienced" } span { "Age 40 — Master" } p { "Begin with a profession at master level." }
                }
            }
            a href="/characters" class="btn btn-secondary" { "Back to adventurers" }
            script src="/static/character-candidates.js?v=3" defer {}
        }
    };
    entry_layout("Choose Your Adventurer", content)
}

pub fn character_candidates_page(
    version: u16,
    seed: &str,
    age_tier: StartingAgeTier,
    candidates: &[StartingCharacterSpec],
    selected: Option<u8>,
    show_inventory: bool,
) -> Result<Markup, adventuresim_world_schema::person_names::NameCatalogError> {
    let presentations = candidates
        .iter()
        .map(CandidatePresentation::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let selected_slot = selected.unwrap_or(0) as usize;
    let candidate = &presentations[selected_slot];
    let spec = &candidates[selected_slot];
    let portraits = presentations
        .iter()
        .enumerate()
        .map(|(slot, candidate)| {
            let profile_href = format!(
                "/characters/candidates?version={version}&seed={seed}&age={}&selected={slot}",
                age_tier.as_str()
            );
            let inventory_href = format!("{profile_href}&view=inventory");
            CharacterPortraitView {
                id: candidate.character.id,
                name: &candidate.character.name,
                alive: true,
                active: false,
                selected: selected_slot == slot,
                href: profile_href,
                title: format!("Inspect {}", candidate.character.name),
                aria_label: format!("Inspect {}", candidate.character.name),
                decoration: Some(html! { span class="candidate-portrait-background" { (&candidates[slot].background) } }),
                badge: None,
                actions: Some(html! {
                    span class="party-portrait-actions" aria-label=(format!("Actions for {}", candidate.character.name)) {
                        a href=(inventory_href)
                            class="party-portrait-action candidate-inventory-action"
                            title=(format!("View {}'s inventory", candidate.character.name))
                            aria-label=(format!("View {}'s inventory", candidate.character.name)) {
                            span class="party-action-icon"
                                style="--party-action-icon: url('/static/icons/game/knapsack.svg')"
                                role="img" aria-label="Inventory" {}
                        }
                    }
                }),
            }
        })
        .collect::<Vec<_>>();
    let medical = MedicalPresentation::default();
    let attributes_title = format!("{}'s attributes", candidate.character.name);
    let skills_title = format!("{}'s skills", candidate.character.name);
    let portraits = character_portrait_overlay("Candidate adventurers", None, &portraits);
    let center_before = html! { nav class="candidate-navigation" { a href="/characters/candidates" { "Change life stage" } span { "Select an adventurer to compare their details" } } span data-candidate-roster data-age-tier=(age_tier.as_str()) hidden {} };
    let center_after = html! {
            @if show_inventory {
                (candidate_inventory_view(spec))
            } @else {
                div class="candidate-background-summary" {
                    p { strong { "Background: " } (&spec.background) }
                }
            }
                form action="/characters/candidates" method="post" class="candidate-play-action" data-candidate-confirm-form {
                    input type="hidden" name="version" value=(version);
                    input type="hidden" name="seed" value=(seed);
                    input type="hidden" name="age" value=(age_tier.as_str());
                    input type="hidden" name="slot" value=(selected_slot);
                    button type="submit" class="btn btn-primary" {
                        "Play as " (&candidate.character.name)
                    }
                }
            script src="/static/character-candidates.js?v=3" defer {}
    };
    let content = character_sheet_markup(CharacterSheetView {
        character: &candidate.character,
        capability: Some(&candidate.capability),
        attributes: Some(&candidate.attributes),
        skills: Some(&candidate.skills),
        limbs: Some(&candidate.limbs),
        personality: Some(&candidate.personality),
        medical: &medical,
        combat_profile: candidate.combat_profile,
        religion_id: candidate.religion_id.as_deref(),
        training_religion_id: None,
        fame: 0.0,
        infamy: 0.0,
        attributes_title: &attributes_title,
        skills_title: &skills_title,
        description: "Adventurer profile",
        can_renounce: false,
        organization_memberships: &candidate.organization_memberships,
        organization_presentation: candidate.organization_presentation.as_ref(),
        organization_minute: adventuresim_world_schema::calendar::StrategicMinute::ZERO,
        physiology_dialog_id: None,
        surgery: None,
        injuries: &[],
        projectiles: &[],
        schedule: None,
        schedule_action: None,
        activity_preview: None,
        activity_location: None,
        professes_religion: false,
        prayer_religion_check: 0.0,
        skill_actions: CharacterSheetActions::default(),
        location_path: "",
        center_before,
        portraits,
        center_after,
        left_after: html! {},
        right_after: html! {},
        after: html! {},
    });

    Ok(entry_layout("Choose Your Adventurer", content))
}

fn candidate_inventory_view(spec: &StartingCharacterSpec) -> Markup {
    html! {
        section class="candidate-inventory-view" data-candidate-inventory {
            header class="candidate-inventory-header" {
                h2 { (item_type_icon("coin")) " Starting inventory" }
                span class="candidate-inventory-purse" {
                    (item_type_icon("coin")) (spec.currency) " coins"
                }
            }
            div class="candidate-inventory-grid" {
                @for item in &spec.inventory {
                    article class="candidate-inventory-item" {
                        (item_type_icon(&item.item_id))
                        span class="candidate-inventory-copy" {
                            strong { (item_display_name(&item.item_id)) }
                            span {
                                "Quantity " (item.quantity)
                                @if let Some(slot) = &item.equipped {
                                    " · Equipped: " (starting_slot_label(*slot))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn starting_slot_label(slot: LoadoutSlot) -> &'static str {
    match slot {
        LoadoutSlot::LeftHand => "left hand",
        LoadoutSlot::RightHand => "right hand",
        LoadoutSlot::LeftArm => "left arm",
        LoadoutSlot::RightArm => "right arm",
        LoadoutSlot::LeftLeg => "left leg",
        LoadoutSlot::RightLeg => "right leg",
        LoadoutSlot::LeftFoot => "left foot",
        LoadoutSlot::RightFoot => "right foot",
        LoadoutSlot::Head => "head",
        LoadoutSlot::Chest => "chest",
        LoadoutSlot::Stomach => "stomach",
    }
}

#[cfg(test)]
mod creation_tests {
    use super::{CandidatePresentation, character_candidates_page};
    use adventuresim_core::organization::StartingProfession;
    use adventuresim_core::starting_character::{StartingAgeTier, StartingItem, roster};

    #[test]
    fn invalid_semantic_identity_fails_candidate_and_page_conversion() {
        use adventuresim_world_schema::{
            Culture,
            person_names::{NameCatalogError, PersonalNameIdentity},
        };
        let mut candidate = adventuresim_core::starting_character::default_character("preview");
        candidate.name_identity = PersonalNameIdentity::authored("\n", Culture::German);
        assert!(matches!(
            CandidatePresentation::try_from(&candidate),
            Err(NameCatalogError::InvalidRenderedName)
        ));
        assert!(matches!(
            character_candidates_page(
                adventuresim_core::starting_character::GENERATOR_VERSION,
                "00112233445566778899aabbccddeeff",
                StartingAgeTier::Young,
                &[candidate],
                Some(0),
                false,
            ),
            Err(NameCatalogError::InvalidRenderedName)
        ));
    }

    #[test]
    fn initial_roster_has_preview_but_no_dialog_or_customization() {
        let candidates = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Young,
        )
        .unwrap();
        let markup = character_candidates_page(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Young,
            &candidates,
            None,
            false,
        )
        .unwrap()
        .into_string();
        assert!(markup.matches("party-portrait").count() >= 5);
        assert!(!markup.contains("prototype-disclaimer"));
        assert!(!markup.contains("role=\"dialog\""));
        assert!(markup.contains("class=\"party-portrait-overlay\""));
        assert!(markup.contains("class=\"party-attributes-list\""));
        assert!(markup.contains("class=\"party-skills-table\""));
        assert!(markup.contains("class=\"party-portrait-actions\""));
        assert!(markup.contains("candidate-inventory-action"));
        assert!(!markup.contains("class=\"schedule-section-heading\""));
        assert!(!markup.contains("data-skill-schedule"));
        assert!(markup.contains("data-candidate-confirm-form"));
        assert!(markup.contains("name=\"slot\" value=\"0\""));
        assert!(!markup.contains("name=\"name\""));
    }

    #[test]
    fn explicit_selection_shows_an_inline_play_action() {
        let candidates = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Adult,
        )
        .unwrap();
        let markup = character_candidates_page(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Adult,
            &candidates,
            Some(2),
            false,
        )
        .unwrap()
        .into_string();
        assert!(!markup.contains("role=\"dialog\""));
        assert!(!markup.contains("aria-modal=\"true\""));
        assert!(!markup.contains("Keep looking"));
        assert!(markup.contains("class=\"candidate-play-action\""));
        assert!(markup.contains("Play as "));
        assert!(markup.contains("name=\"slot\" value=\"2\""));
        assert_eq!(markup.matches("data-character-alive=\"true\"").count(), 10);
        assert!(markup.contains("name=\"age\" value=\"adult\""));
        assert!(!markup.contains("data-candidate-package"));
        assert!(markup.contains("organization-identity-control is-readonly"));
        assert!(markup.contains("religion-identity-control"));
        assert!(markup.contains("candidate-inventory-action"));
    }

    #[test]
    fn candidate_inventory_opens_from_the_portrait_without_listing_the_package_on_profile() {
        let candidates = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Adult,
        )
        .unwrap();
        let markup = character_candidates_page(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Adult,
            &candidates,
            Some(2),
            true,
        )
        .unwrap()
        .into_string();
        assert!(markup.contains("data-candidate-inventory"));
        assert!(markup.contains("Starting inventory"));
        assert!(markup.contains("candidate-inventory-item"));
        assert!(markup.contains("view=inventory"));
        assert!(!markup.contains("Package:"));
        assert!(markup.contains("party-attributes-list"));
        assert!(markup.contains("party-skills-table"));
    }

    #[test]
    fn preview_capabilities_use_equipped_items_and_professional_skill_values() {
        let mut young = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Young,
        )
        .unwrap()
        .remove(0);
        young.inventory.push(StartingItem {
            item_id: "longbow".into(),
            quantity: 1,
            equipped: None,
        });
        let preview = CandidatePresentation::try_from(&young).unwrap();
        assert!(!preview.capability.ranged);

        let mut adult = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Adult,
        )
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.profession == Some(StartingProfession::Herbalist))
        .unwrap();
        adult.skills.physiology = 100.0;
        let preview = CandidatePresentation::try_from(&adult).unwrap();
        assert!(preview.capability.physiology > 0.0);
        assert_eq!(preview.capability.surgery, 0.0);
        assert!(preview.capability.weapon_precision > 0.0);

        adult.skills.knife = 10_000.0;
        adult.skills.tailoring = 10_000.0;
        assert_eq!(
            CandidatePresentation::try_from(&adult)
                .unwrap()
                .capability
                .surgery,
            0.0,
            "correlated crafts must not unlock a trained skill without direct Surgery study"
        );

        adult.skills.surgery = 100.0;
        let correlated = CandidatePresentation::try_from(&adult)
            .unwrap()
            .capability
            .surgery;
        adult.skills.knife = 0.0;
        adult.skills.tailoring = 0.0;
        let direct_only = CandidatePresentation::try_from(&adult)
            .unwrap()
            .capability
            .surgery;
        assert!(correlated > direct_only);
    }

    #[test]
    fn candidate_surgery_capability_uses_the_weighted_governing_aptitude() {
        let mut candidate = roster(
            adventuresim_core::starting_character::GENERATOR_VERSION,
            "00112233445566778899aabbccddeeff",
            StartingAgeTier::Young,
        )
        .unwrap()
        .remove(0);
        candidate.attributes.intelligence = 4.0;
        candidate.attributes.instinct = 1.0;
        candidate.attributes.agility = 3.5;
        candidate.skills.surgery = adventuresim_core::skill::Skill::Surgery.hours_for_rank(4.0);
        candidate.skills.knife = 0.0;
        candidate.skills.tailoring = 0.0;

        let surgery = CandidatePresentation::try_from(&candidate)
            .unwrap()
            .capability
            .surgery;
        assert!((surgery - 3.15).abs() < 0.001);
    }

    #[test]
    fn preview_membership_dues_use_the_same_initial_interval_semantics() {
        let candidates = [StartingAgeTier::Adult, StartingAgeTier::Old]
            .into_iter()
            .flat_map(|tier| {
                roster(
                    adventuresim_core::starting_character::GENERATOR_VERSION,
                    "00112233445566778899aabbccddeeff",
                    tier,
                )
                .unwrap()
            });
        let candidate = candidates
            .into_iter()
            .find(|candidate| {
                candidate.organization.as_ref().is_some_and(|organization| {
                    adventuresim_core::organization::organization(&organization.organization_id)
                        .is_some_and(|definition| definition.dues.is_some())
                })
            })
            .unwrap();
        let preview = CandidatePresentation::try_from(&candidate).unwrap();
        let membership = &preview.organization_memberships[0];
        let definition =
            adventuresim_core::organization::organization(&membership.organization_id).unwrap();
        let expected = adventuresim_world_schema::calendar::StrategicMinute::ZERO
            .saturating_add_days(u64::from(definition.dues.as_ref().unwrap().interval_days));
        assert_eq!(
            membership.joined_minute,
            adventuresim_stdb_client::StrategicMinute { minutes: 0 }
        );
        assert_eq!(
            membership.dues_paid_through_minute,
            adventuresim_stdb_client::StrategicMinute {
                minutes: expected.get(),
            }
        );
    }
}
