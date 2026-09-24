//! Opt-in rendered fixtures for desktop and narrow-browser interface review.
//! No database, runtime authority, or player records are used.

use super::*;
use crate::medical::{ChartReadingPresentation, MedicalPresentation};
use crate::spacetimedb::{BodyRegion, CharacterView, InventoryItem, SettlementCategory};
use crate::templates::character::{
    character_candidates_bootstrap_page, character_candidates_page, characters_list_page,
};
use adventuresim_core::equipment::EncumbranceSummary;
use adventuresim_core::starting_character::{GENERATOR_VERSION, StartingAgeTier, roster};
use maud::{Markup, html};

fn adventurer(id: u64, name: &str) -> CharacterView {
    CharacterView {
        id,
        name: name.into(),
        xp: 0,
        level: 1,
        current_settlement_id: Some("goslar".into()),
        current_case_site_id: None,
        party_id: Some("review-party".into()),
        age_years: 24,
        alive: true,
        temporary: false,
        social_notification_count: 0,
        automatic_social_chat_enabled: false,
    }
}

fn location() -> LocationView {
    LocationView {
        kind: crate::location_urls::LocationKind::Settlement,
        id: "goslar".into(),
        name: "Goslar".into(),
        religion_id: Some("western_church".into()),
        category: Some(SettlementCategory::City),
        economy: None,
        active_building: Some("inn".into()),
    }
}

fn inventory(location: &LocationView, characters: &[CharacterView]) -> Markup {
    let items: Vec<_> = ["sword", "bandage", "soap", "cooking_pot", "longbow"]
        .into_iter()
        .enumerate()
        .map(|(index, item)| InventoryItem {
            id: index as u64 + 1,
            character_id: characters[0].id,
            item_id: item.into(),
            quantity: 1,
        })
        .collect();
    party_inventory_page(
        location,
        &characters[1],
        &items,
        &characters[0],
        &items,
        &[],
        &[],
        characters,
        None,
        None,
        &[],
        &[],
        EncumbranceSummary::new(8.0, 60.0),
        EncumbranceSummary::new(12.0, 60.0),
    )
}

fn injured_stats(examined: bool) -> Markup {
    let attributes = crate::spacetimedb::CharacterAttributes {
        character_id: 7,
        endurance: 3.0,
        immunity: 3.0,
        gut: 3.0,
        intelligence: 3.0,
        instinct: 3.0,
        eyesight: 3.0,
        hearing: 3.0,
        left_arm_strength: 3.0,
        right_arm_strength: 3.0,
        left_leg_strength: 3.0,
        right_leg_strength: 3.0,
        left_arm_agility: 3.0,
        right_arm_agility: 3.0,
        left_leg_agility: 3.0,
        right_leg_agility: 3.0,
    };
    let limbs = crate::spacetimedb::CharacterLimbs {
        character_id: 7,
        head_health: 1.0,
        chest_health: 0.75,
        stomach_health: 1.0,
        left_arm_health: 1.0,
        right_arm_health: 1.0,
        left_leg_health: 0.4,
        right_leg_health: 1.0,
    };
    let medical = MedicalPresentation {
        regional_humours: examined.then_some(
            [crate::medical::HumourVitals {
                sanguine: 0.12,
                phlegmatic: 0.12,
                choleric: 0.12,
                melancholic: 0.12,
            }; 7],
        ),
        concealed_other: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2],
        ..Default::default()
    };
    crate::templates::entry_layout(
        "Condition review",
        html! {
            aside class="left-sidebar" {
                (character_health::party_attributes_rail("Known condition", Some(&attributes), Some(&limbs), &medical, None, Some(("/review/surgery/__limb__", None)), &[], &[]))
            }
            main class="center-content" {} aside class="right-sidebar" {}
        },
    )
}

#[test]
fn export_interface_review_fixtures() {
    let Ok(directory) = std::env::var("UX_CAPTURE_DIR") else {
        return;
    };
    let directory = std::path::Path::new(&directory);
    std::fs::create_dir_all(directory).unwrap();
    let place = location();
    let characters = [
        adventurer(7, "Anna von Winterfeld"),
        adventurer(8, "Benedikt Adler"),
    ];
    let seed = "00112233445566778899aabbccddeeff";
    let candidates = roster(GENERATOR_VERSION, seed, StartingAgeTier::Adult).unwrap();
    let medical = MedicalPresentation {
        readings: vec![ChartReadingPresentation {
            minute: 1440,
            physiology_band: 2,
            observation_minutes: 60,
            humour_deviations_bps: [[200, -100, 0, 0]; 7],
            possible_diseases: vec![],
            known_interventions: vec![],
            confidence_bps: 4500,
        }],
        ..MedicalPresentation::default()
    };
    let surgery = surgery_dialog(
        &place,
        &characters[0],
        &characters[0],
        &[],
        &[],
        BodyRegion::Head,
        3,
        0,
        0,
        1,
        0,
        None,
        1.0,
    );
    let notebook =
        character_health::physiology_dialog(&medical, "review-notebook", &characters[0].name);
    let party = crate::spacetimedb::PartyView {
        id: "review-party".into(),
        gateway_bucket: 0,
        name: "Adventurers".into(),
        leader_id: 7,
        current_settlement_id: Some("goslar".into()),
        current_case_site_id: None,
        active_contract_id: None,
        is_solo: false,
        camp_fatigue_percent: 50,
        walking_minutes_per_day: 480,
        travel_at_night: false,
        journey_start_minute_of_day: 480,
        wilderness_canonical_anchor_minute: None,
        wilderness_elapsed_minutes: 0,
        camp_destination: None,
        camp_remaining_minutes: 0,
        physiology_target: 0.0,
        command_target: 0.0,
        religion_target: 0.0,
    };
    let recruitment =
        crate::templates::recruitment::recruitment_panel(&party, 7, &[], &[], Default::default());
    let rest = rest_service_menu(
        "Inn",
        "goslar",
        RestServiceKind::Inn,
        Some(1440),
        None,
        SoapRestPreview {
            total_units: 1,
            personal_units: 1,
            shared_units: 0,
            available_units: 1,
            alcohol_available: false,
            alcohol_will_be_consumed: false,
        },
    );
    let pages = [
        ("injured-stats", injured_stats(false)),
        ("examined-stats", injured_stats(true)),
        ("recruitment", place.render_layout("Recruitment", html! { main class="center-content" { (recruitment) } }, Some(&characters[0].name))),
        ("rest", place.render_layout("Inn", html! { aside class="left-sidebar" { (rest) } main class="center-content" {} aside class="right-sidebar" {} }, Some(&characters[0].name))),
        (
            "life-stage",
            character_candidates_bootstrap_page(GENERATOR_VERSION),
        ),
        (
            "candidates",
            character_candidates_page(
                GENERATOR_VERSION,
                seed,
                StartingAgeTier::Adult,
                &candidates,
                Some(0),
                false,
            ),
        ),
        (
            "roster",
            characters_list_page(&characters, &[], Some(characters[0].id)),
        ),
        ("inventory", inventory(&place, &characters)),
        (
            "surgery",
            place.render_layout(
                "Treatment",
                html! { main class="center-content" { (surgery) } },
                Some(&characters[0].name),
            ),
        ),
        (
            "notebook",
            place.render_layout(
                "Physician notebook",
                html! { main class="center-content" { (notebook) } },
                Some(&characters[0].name),
            ),
        ),
        (
            "cooking",
            fireplace_page(
                "Cooking",
                "/",
                "/review/cooking",
                "/review/rest",
                &characters[0],
                "personal",
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                None,
                None,
                &[],
                &[],
                1440,
                |content| place.render_layout("Cooking", content, Some(&characters[0].name)),
            ),
        ),
    ];
    for (name, page) in pages {
        let html = page.into_string();
        assert!(html.contains("id=\"strategic-page\""));
        std::fs::write(directory.join(format!("{name}.html")), html).unwrap();
    }
}
