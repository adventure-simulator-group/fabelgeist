//! HTML residence choices reference authoritative generated homes.
use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "the residence page boundary composes independent character and residence projections"
)]
pub fn settlement_residence_page(
    settlement: &SettlementView,
    active_character: &CharacterView,
    party_members: &[CharacterView],
    logged_in_as: Option<&str>,
    offers: &[AvailableResidenceProperty],
    holdings: &[BackendCharacterResidenceStatus],
    relationship: Option<&RelationshipPresentation>,
    can_rest_at_home: bool,
    notice: Option<&str>,
) -> Markup {
    let panel = residence_offer_panel(
        settlement,
        active_character.id,
        offers,
        holdings,
        relationship,
        notice,
    );
    settlement_resident_location_page_with_panel(
        settlement,
        active_character,
        party_members,
        "residences",
        logged_in_as,
        Some(panel),
        can_rest_at_home,
    )
}

fn child_stage_label(stage: ChildStage) -> &'static str {
    match stage {
        ChildStage::EarlyChildhood => "early childhood",
        ChildStage::MiddleChildhood => "middle childhood",
        ChildStage::Adolescence => "adolescence",
        ChildStage::Adult => "adult",
    }
}

fn child_stage_id(stage: ChildStage) -> &'static str {
    match stage {
        ChildStage::EarlyChildhood => "early",
        ChildStage::MiddleChildhood => "middle",
        ChildStage::Adolescence => "adolescent",
        ChildStage::Adult => "adult",
    }
}

fn child_activity_presentation(
    stage: ChildStage,
    focus: ChildActivityFocus,
) -> (&'static str, &'static str) {
    if stage == ChildStage::EarlyChildhood {
        return ("bed", "care and rest");
    }
    match focus {
        ChildActivityFocus::Play => ("sun", "play"),
        ChildActivityFocus::Study => ("open-book", "study"),
        ChildActivityFocus::HouseholdHelp => ("hammer-sickle", "household help"),
        ChildActivityFocus::SocialLearning => ("person", "social learning"),
    }
}

fn residence_meter(icon: &str, class_name: &str, label: &str, value: u32, maximum: u32) -> Markup {
    let percent = value.saturating_mul(100) / maximum.max(1);
    html! {
        span class=(format!("residence-meter residence-meter-{class_name}"))
            role="meter" aria-valuemin="0" aria-valuemax=(maximum) aria-valuenow=(value)
            aria-valuetext=(label) title=(label) data-strategic-tooltip=(label) {
            (decorative_game_icon(icon))
            span class="residence-meter-track" aria-hidden="true" {
                span class="residence-meter-fill" style=(format!("width: {percent}%")) {}
            }
        }
    }
}

fn household_progress(icon: &str, class_name: &str, label: &str, percent: u64) -> Markup {
    html! {
        span class=(format!("household-progress household-{class_name}"))
            role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow=(percent.min(100))
            aria-valuetext=(label) title=(label) data-strategic-tooltip=(label) {
            (decorative_game_icon(icon))
            span class="household-progress-track" aria-hidden="true" {
                span class="household-progress-fill" style=(format!("width: {}%", percent.min(100))) {}
            }
        }
    }
}

fn residence_tier_label(tier: HousingTier) -> &'static str {
    match tier {
        HousingTier::Cheap => "Cheap",
        HousingTier::Moderate => "Moderate",
        HousingTier::Fancy => "Fancy",
    }
}

fn residence_tier_id(tier: HousingTier) -> &'static str {
    match tier {
        HousingTier::Cheap => "cheap",
        HousingTier::Moderate => "moderate",
        HousingTier::Fancy => "fancy",
    }
}

fn residence_offer_panel(
    settlement: &SettlementView,
    active_character_id: u64,
    offers: &[AvailableResidenceProperty],
    holdings: &[BackendCharacterResidenceStatus],
    relationship: Option<&RelationshipPresentation>,
    notice: Option<&str>,
) -> Markup {
    let max_rent = offers
        .iter()
        .map(|offer| offer.rent_per_period)
        .max()
        .unwrap_or(1);
    let max_purchase = offers
        .iter()
        .map(|offer| offer.purchase_price)
        .max()
        .unwrap_or(1);
    let max_owner_cost = offers
        .iter()
        .map(|offer| offer.owner_maintenance_per_period + offer.property_tax_per_period)
        .max()
        .unwrap_or(1);
    let max_leisure = offers
        .iter()
        .map(|offer| u32::from(offer.leisure_morale_basis_points))
        .max()
        .unwrap_or(1);
    sidebar_section(
        "Residences",
        html! {
            @if let Some(notice) = notice {
                p class="residence-notice" role="status" { (notice) }
            }
            (holdings_panel(settlement, active_character_id, holdings))
            (family_panel(relationship))
            div class="residence-offer-grid" {
            @for offer in offers {
                @let tier = residence_tier_label(offer.tier);
                @let owner_cost = offer.owner_maintenance_per_period + offer.property_tax_per_period;
                @let rent_label = format!("Rent: {} coin every 30 days", offer.rent_per_period);
                @let purchase_label = format!("Purchase: {} coin", offer.purchase_price);
                @let upkeep_label = format!("Owner upkeep: {} maintenance plus {} property tax every 30 days", offer.owner_maintenance_per_period, offer.property_tax_per_period);
                @let leisure_label = format!("Leisure morale bonus: {} percent", format_morale_percent(offer.leisure_morale_basis_points));
                @let rent_confirmation = format!("return confirm('Rent this {tier} residence for {} coin every 30 days?')", offer.rent_per_period);
                @let buy_confirmation = format!("return confirm('Buy this {tier} residence for {} coin? Ongoing owner costs are {} coin every 30 days.')", offer.purchase_price, owner_cost);
                article class="residence-offer" data-residence-tier=(residence_tier_id(offer.tier)) {
                    div class="residence-offer-heading" { (decorative_game_icon("house")) strong { (tier) " home " (offer.building_id) } }
                    div class="residence-meter-list" {
                        (residence_meter("bed", "rent", &rent_label, offer.rent_per_period, max_rent))
                        (residence_meter("coins", "purchase", &purchase_label, offer.purchase_price, max_purchase))
                        (residence_meter("hammer-nails", "upkeep", &upkeep_label, owner_cost, max_owner_cost))
                        (residence_meter("sun", "leisure", &leisure_label, u32::from(offer.leisure_morale_basis_points), max_leisure))
                    }
                    div class="residence-meter-actions" {
                    form action=(crate::location_urls::patterns::CHANGE_RESIDENCE.url([&settlement.id, &("rent"), &(residence_tier_id(offer.tier))])) method="post" onsubmit=(&rent_confirmation) {
                        input type="hidden" name="property_id" value=(&offer.property_id);
                        button type="submit" class="residence-icon-action" title=(&rent_label) aria-label=(format!("Rent {tier}. {rent_label}")) {
                            (decorative_game_icon("bed")) span class="sr-only" { "Rent " (tier) }
                        }
                    }
                    form action=(crate::location_urls::patterns::CHANGE_RESIDENCE.url([&settlement.id, &("buy"), &(residence_tier_id(offer.tier))])) method="post" onsubmit=(&buy_confirmation) {
                        input type="hidden" name="property_id" value=(&offer.property_id);
                        button type="submit" class="residence-icon-action" title=(&purchase_label) aria-label=(format!("Buy {tier}. {purchase_label}. {upkeep_label}")) {
                            (decorative_game_icon("coins")) span class="sr-only" { "Buy " (tier) }
                        }
                    }
                    }
                }
            }
            }
        },
    )
}

fn holdings_panel(
    settlement: &SettlementView,
    active_character_id: u64,
    holdings: &[BackendCharacterResidenceStatus],
) -> Markup {
    html! {
            @if holdings.is_empty() {
                p { "No residence holdings." }
            } @else {
                div class="residence-holding-list" {
                    @for holding in holdings {
                        @let owns_holding = holding.holder_character_id == active_character_id;
                        @let tier = residence_tier_label(holding.tier);
                        @let tenure = match holding.tenure { ResidenceTenure::Renter => "Rental", ResidenceTenure::Owner => "Owned property" };
                        @let payment_label = if holding.active { format!("Next payment {}", format_residence_date(StrategicMinute::new(holding.next_due_minute.minutes))) } else { format!("Payment overdue since {}", format_residence_date(StrategicMinute::new(holding.next_due_minute.minutes))) };
                        article class="residence-holding" data-holding-id=(&holding.holding_id) {
                            div class="residence-holding-heading" {
                                (decorative_game_icon("house"))
                                strong { (tier) }
                                span { (&holding.settlement_id) }
                                div class="residence-holding-states" role="group" aria-label=(format!("{tier} residence status")) {
                                    span class="residence-holding-state" role="img" title=(if owns_holding { tenure } else { "Household home" }) aria-label=(if owns_holding { tenure } else { "Household home" }) {
                                        (decorative_game_icon(if owns_holding && holding.tenure == ResidenceTenure::Renter { "bed" } else { "house" }))
                                    }
                                    @if holding.primary {
                                        span class="residence-holding-state" role="img" title="Designated home" aria-label="Designated home" { (decorative_game_icon("crown")) }
                                    }
                                    @if holding.occupied {
                                        span class="residence-holding-state" role="img" title="Occupied" aria-label="Occupied" { (decorative_game_icon("person")) }
                                    }
                                    span class=(if holding.active { "residence-holding-state" } else { "residence-holding-state residence-holding-state-inactive" })
                                        role="img" title=(&payment_label) aria-label=(&payment_label) data-strategic-tooltip=(&payment_label) {
                                        (decorative_game_icon(if holding.active { "calendar" } else { "cross-mark" }))
                                    }
                                }
                            }
                            div class="residence-holding-actions" {
                            @if owns_holding && holding.tenure == ResidenceTenure::Owner && !holding.active {
                                form action=(crate::location_urls::patterns::CHANGE_RESIDENCE.url([&settlement.id, &("recover"), &("current")])) method="post" {
                                    input type="hidden" name="holding_id" value=(&holding.holding_id);
                                    button type="submit" class="residence-icon-action" title="Recover owned home" aria-label="Recover owned home" {
                                        (decorative_game_icon("hammer-nails")) span class="sr-only" { "Recover owned home" }
                                    }
                                }
                            }
                            @if owns_holding && holding.active && !holding.primary && holding.settlement_id == settlement.id {
                                form action=(crate::location_urls::patterns::CHANGE_RESIDENCE.url([&settlement.id, &("designate"), &("current")])) method="post" {
                                    input type="hidden" name="holding_id" value=(&holding.holding_id);
                                    button type="submit" class="residence-icon-action" title="Designate as home" aria-label="Designate as home" {
                                        (decorative_game_icon("crown")) span class="sr-only" { "Designate as home" }
                                    }
                                }
                            }
                            @if owns_holding {
                                form action=(crate::location_urls::patterns::CHANGE_RESIDENCE.url([&settlement.id, &("relinquish"), &("current")])) method="post"
                                    onsubmit="return confirm('Relinquish this property? This cannot be undone.')" {
                                    input type="hidden" name="holding_id" value=(&holding.holding_id);
                                    button type="submit" class="residence-icon-action residence-icon-action-danger" title="Relinquish property" aria-label="Relinquish property" {
                                        (decorative_game_icon("cross-mark")) span class="sr-only" { "Relinquish property" }
                                    }
                                }
                            }
                            }
                        }
                    }
                }
            }
    }
}

fn family_panel(relationship: Option<&RelationshipPresentation>) -> Markup {
    html! {
            @if let Some(relationship) = relationship {
                div class="household-visual" role="group" aria-label="Household and family" {
                @if let Some(spouse_name) = &relationship.spouse_name {
                    div class="household-member" aria-label=(format!("Spouse: {spouse_name}")) title=(format!("Spouse: {spouse_name}")) {
                        (decorative_game_icon("heart-beats")) span class="household-member-name" { (spouse_name) }
                    }
                }
                @if let Some(partner_name) = &relationship.courtship_partner_name {
                    @let courtship_kind = relationship.courtship_kind.unwrap_or(CourtshipKind::Informal);
                    @let informal = courtship_kind == CourtshipKind::Informal;
                    @let courtship_kind_label = match courtship_kind { CourtshipKind::Formal => "formal", CourtshipKind::Informal => "informal" };
                    @let courtship_visibility = if !informal { "; formal and public" } else if relationship.courtship_exposed { "; known to family" } else { "; private" };
                    @let courtship_icon = if !informal { "rose" } else if relationship.courtship_exposed { "eye-target" } else { "lockpicks" };
                    @let courtship_label = format!("{courtship_kind_label} courtship with {partner_name}{courtship_visibility}");
                    div class="household-member" title=(&courtship_label) aria-label=(&courtship_label) {
                        (decorative_game_icon("rose")) span class="household-member-name" { (partner_name) }
                        span class="household-member-state" aria-hidden="true" {
                            (decorative_game_icon(courtship_icon))
                        }
                    }
                }
                @if let Some(wedding) = &relationship.wedding {
                    @let wedding_label = format!("Wedding {}, {} days remaining", wedding.date_label, wedding.days_remaining);
                    @let wedding_progress = DAYS_PER_YEAR.saturating_sub(wedding.days_remaining.min(DAYS_PER_YEAR)) * 100 / DAYS_PER_YEAR;
                    (household_progress("calendar", "wedding", &wedding_label, wedding_progress))
                }
                @if let Some(days) = relationship.pregnancy_due_days {
                    @let pregnancy_label = format!("Expected birth in about {days} days");
                    @let pregnancy_progress = 270_u64.saturating_sub(days.min(270)) * 100 / 270;
                    (household_progress("stomach", "pregnancy", &pregnancy_label, pregnancy_progress))
                }
                @for child in &relationship.children {
                    @let stage = child_stage_label(child.stage);
                    @let stage_id = child_stage_id(child.stage);
                    @let (focus_icon, focus) = child_activity_presentation(child.stage, child.focus);
                    @let state = if !child.alive { "deceased" } else if child.adult_playable { "adult and available in the character roster" } else { stage };
                    @let label = format!("Child: {}; {state}; focus: {focus}; maturity {} percent", child.name, child.maturity_basis_points / 100);
                    div class=(format!("household-child household-child-stage-{stage_id}")) aria-label=(&label) title=(&label) data-strategic-tooltip=(&label) {
                        div class="household-member" {
                            (decorative_game_icon("person")) span class="household-member-name" { (&child.name) }
                            span class="household-member-state" aria-hidden="true" {
                                (decorative_game_icon(if !child.alive { "death-skull" } else if child.adult_playable { "crown" } else { focus_icon }))
                            }
                        }
                        (household_progress("person", "maturity", &label, u64::from(child.maturity_basis_points) / 100))
                    }
                }
                }
            }
    }
}

fn format_morale_percent(basis_points: u16) -> String {
    let whole = basis_points / 100;
    let fraction = basis_points % 100;
    if fraction == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{fraction:02}")
    }
}

fn format_residence_date(minute: StrategicMinute) -> String {
    let year = minute.calendar_year();
    format!("year {year}, day {}", minute.day_of_year())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::settlement::test_support::*;
    #[test]
    fn residence_panel_compares_three_tiers_and_names_family_without_raw_time_or_ids() {
        let settlement = settlement();
        let offers = [
            HousingTier::Cheap,
            HousingTier::Moderate,
            HousingTier::Fancy,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, tier)| AvailableResidenceProperty {
            property_id: format!("home-{index}"),
            building_id: index as u64 + 1,
            resident_capacity: 8,
            available_from_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
            settlement_id: settlement.id.clone(),
            tier,
            purchase_price: 100 + index as u32 * 100,
            rent_per_period: 10 + index as u32 * 10,
            owner_maintenance_per_period: 3 + index as u32,
            property_tax_per_period: 2 + index as u32,
            leisure_morale_basis_points: 40 + index as u16 * 40,
        })
        .collect::<Vec<_>>();
        let relationship = RelationshipPresentation {
            spouse_name: Some("Anna".into()),
            courtship_partner_name: Some("Bea".into()),
            courtship_kind: Some(CourtshipKind::Formal),
            courtship_exposed: false,
            wedding: Some(WeddingPresentation {
                days_remaining: DAYS_PER_YEAR,
                date_label: "year 1546, day 120".into(),
            }),
            pregnancy_due_days: Some(270),
            children: vec![ChildPresentation {
                name: "Elsa".into(),
                stage: ChildStage::MiddleChildhood,
                focus: ChildActivityFocus::Study,
                maturity_basis_points: 4_000,
                adult_playable: false,
                alive: true,
            }],
        };
        let holdings = vec![
            BackendCharacterResidenceStatus {
                character_id: 1,
                property_id: "physical-home".into(),
                holding_id: "holding-primary".into(),
                holder_character_id: 1,
                settlement_id: settlement.id.clone(),
                tier: HousingTier::Moderate,
                tenure: ResidenceTenure::Owner,
                active: true,
                primary: true,
                occupied: true,
                acquired_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                last_billed_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                next_due_minute: adventuresim_stdb_client::StrategicMinute { minutes: 43_200 },
            },
            BackendCharacterResidenceStatus {
                character_id: 1,
                property_id: "physical-home".into(),
                holding_id: "holding-dormant".into(),
                holder_character_id: 1,
                settlement_id: "elsewhere".into(),
                tier: HousingTier::Cheap,
                tenure: ResidenceTenure::Owner,
                active: false,
                primary: false,
                occupied: false,
                acquired_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                last_billed_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                next_due_minute: adventuresim_stdb_client::StrategicMinute { minutes: 43_200 },
            },
            BackendCharacterResidenceStatus {
                character_id: 1,
                property_id: "physical-home".into(),
                holding_id: "holding-household".into(),
                holder_character_id: 2,
                settlement_id: settlement.id.clone(),
                tier: HousingTier::Fancy,
                tenure: ResidenceTenure::Owner,
                active: true,
                primary: false,
                occupied: true,
                acquired_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                last_billed_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                next_due_minute: adventuresim_stdb_client::StrategicMinute { minutes: 43_200 },
            },
        ];
        let markup = residence_offer_panel(
            &settlement,
            1,
            &offers,
            &holdings,
            Some(&relationship),
            None,
        )
        .into_string();

        if let Ok(path) = std::env::var("RESIDENCE_REVIEW_OUTPUT") {
            std::fs::write(path, &markup).unwrap();
        }

        assert_eq!(markup.matches("name=\"property_id\"").count(), 6);
        assert!(markup.contains("name=\"property_id\" value=\"home-1\""));
        assert_eq!(markup.matches("class=\"residence-offer\"").count(), 3);
        assert_eq!(markup.matches("class=\"residence-holding\"").count(), 3);
        assert!(markup.contains("name=\"holding_id\" value=\"holding-primary\""));
        assert!(markup.contains("name=\"holding_id\" value=\"holding-dormant\""));
        assert!(!markup.contains("name=\"holding_id\" value=\"holding-household\""));
        assert!(markup.contains("Fancy"));
        assert!(markup.contains("Household home"));
        assert!(markup.contains("Recover owned home"));
        for tier in ["cheap", "moderate", "fancy"] {
            assert!(markup.contains(&format!("data-residence-tier=\"{tier}\"")));
        }
        assert_eq!(markup.matches("class=\"residence-meter-list\"").count(), 3);
        assert_eq!(markup.matches("residence-meter-leisure").count(), 3);
        assert!(markup.contains("data-strategic-tooltip=\"Owner upkeep:"));
        assert!(markup.contains("Leisure morale bonus:"));
        assert!(markup.contains("onsubmit=\"return confirm("));
        assert!(markup.contains("Rent this Cheap residence"));
        assert!(markup.contains("Relinquish this property? This cannot be undone."));
        assert!(markup.contains("Spouse: Anna"));
        assert!(markup.contains("Child: Elsa"));
        assert!(markup.contains("middle childhood; focus: study"));
        assert!(markup.contains("household-child-stage-middle"));
        assert!(markup.contains("household-progress household-maturity"));
        assert!(markup.contains("role=\"meter\""));
        assert!(!markup.contains("Elsa, age"));
        assert!(markup.contains("formal courtship with Bea; formal and public"));
        assert!(!markup.contains("formal courtship with Bea; private"));
        assert!(markup.contains("household-progress household-wedding"));
        assert!(markup.contains("household-progress household-pregnancy"));
        assert!(markup.contains("role=\"meter\""));
        assert!(markup.contains("aria-valuenow="));
        assert!(markup.contains("role=\"img\""));
        assert!(markup.contains("aria-label=\"Spouse: Anna\""));
        assert!(markup.contains(
            "aria-label=\"Child: Elsa; middle childhood; focus: study; maturity 40 percent\""
        ));
        assert!(!markup.contains("character #"));
        assert!(!markup.contains("strategic minute"));
    }

    #[test]
    fn early_child_activity_is_care_regardless_of_future_focus() {
        assert_eq!(
            child_activity_presentation(ChildStage::EarlyChildhood, ChildActivityFocus::Study),
            ("bed", "care and rest")
        );
        assert_eq!(
            child_activity_presentation(ChildStage::MiddleChildhood, ChildActivityFocus::Study),
            ("open-book", "study")
        );
    }
}
