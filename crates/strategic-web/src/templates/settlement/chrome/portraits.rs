//! Attached portrait navigation for characters and party inventory.
use crate::spacetimedb::CharacterView;
use crate::templates::game_icon;
use maud::{Markup, html};

pub(crate) struct CharacterPortraitView<'a> {
    pub id: u64,
    pub name: &'a str,
    pub alive: bool,
    pub active: bool,
    pub selected: bool,
    pub href: String,
    pub title: String,
    pub aria_label: String,
    pub decoration: Option<Markup>,
    pub badge: Option<Markup>,
    pub actions: Option<Markup>,
}

pub(crate) fn character_portrait_overlay(
    label: &str,
    inventory: Option<Markup>,
    members: &[CharacterPortraitView<'_>],
) -> Markup {
    html! {
        @if !members.is_empty() {
            div class="party-portrait-overlay" aria-label=(label) {
                div data-party-portrait-members {
                    @if let Some(inventory) = inventory {
                        (inventory)
                    }
                    @for member in members {
                        div class=(format!("scene-interactable scene-interactable--person party-portrait{}{}", if member.selected { " active" } else { "" }, if !member.alive { " dead" } else { "" }))
                            data-character-id=(member.id)
                            data-character-alive=(member.alive)
                            data-active-character[member.active]
                            title=(member.name) {
                            a class="party-portrait-select"
                                href=(&member.href)
                                title=(&member.title)
                                aria-label=(&member.aria_label) {
                                @if let Some(decoration) = &member.decoration {
                                    (decoration)
                                }
                                span class="scene-interactable-visual party-portrait-initial" {
                                    span class="party-portrait-face" { (member.name.chars().next().unwrap_or('?')) }
                                    @if let Some(badge) = &member.badge {
                                        (badge)
                                    }
                                }
                                span class="scene-interactable-label party-portrait-name" { (member.name) @if !member.alive { " (dead)" } }
                            }
                            @if let Some(actions) = &member.actions {
                                (actions)
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn party_portrait_overlay(
    party_members: &[CharacterView],
    active_character: Option<&CharacterView>,
    location_path: &str,
    selected_character_id: Option<u64>,
) -> Markup {
    let members: Vec<&CharacterView> = if party_members.is_empty() {
        active_character.into_iter().collect()
    } else {
        party_members.iter().collect()
    };

    let inventory = active_character.map(|_| {
        html! {
            div class="scene-interactable scene-interactable--fixture party-portrait party-inventory-portrait" title="Party inventory" {
                a class="party-portrait-select" data-portrait-tab="party-inventory" href=(format!("{}/party-inventory", location_path)) {
                    span class="scene-interactable-visual party-portrait-initial party-chest-face" { (game_icon("Party inventory", "knapsack")) }
                    span class="party-portrait-name" { "Party inventory" }
                }
            }
        }
    });
    let portraits = members
        .into_iter()
        .map(|member| {
            let is_active = active_character.is_some_and(|character| character.id == member.id);
            let notified = member.alive && member.social_notification_count > 0;
            let persistently_notified = notified && !member.automatic_social_chat_enabled;
            let inspection_href = if is_active {
                format!("{}/party/{}", location_path, member.id)
            } else {
                format!("{}/party/{}/stats", location_path, member.id)
            };
            let actions = Some(html! {
                    nav class="portrait-tabs" aria-label=(format!("Views for {}", member.name)) {
                            a class="portrait-tab" href=(&inspection_href) data-portrait-tab="profile"
                                title="Profile" aria-label=(format!("Profile of {}", member.name)) {
                                (game_icon("Profile", "person"))
                            }
                            @if member.alive && active_character.is_some_and(|character| character.alive) {
                            a href=(format!("{}/party/{}/social", location_path, member.id))
                                class=(format!("portrait-tab party-social-action{}", if persistently_notified { " party-social-notified" } else { "" }))
                                data-portrait-tab="conversation" title=(if notified { format!("Open {}'s Recent Tidings ({} morale concerns)", member.name, member.social_notification_count) } else { format!("Talk to {}", member.name) })
                                aria-label=(if notified { format!("Open conversation with {} to Recent Tidings; {} unaddressed morale concerns", member.name, member.social_notification_count) } else { format!("Open conversation with {}", member.name) }) {
                                span class="party-action-icon"
                                    style="--party-action-icon: url('/static/icons/game/conversation.svg')"
                                    aria-hidden="true" {}
                                @if notified {
                                    span class="party-social-notification" aria-hidden="true" {
                                        (member.social_notification_count)
                                    }
                                }
                            }
                            a href=(format!("{}/party/{}/inventory", location_path, member.id))
                                class="portrait-tab" data-portrait-tab="inventory" aria-label=(format!("Inventory of {}", member.name))
                                title=(if is_active { "Open inventory and discard items".to_string() } else { format!("Compare inventory with {}", member.name) }) {
                                span class="party-action-icon"
                                    style="--party-action-icon: url('/static/icons/game/knapsack.svg')"
                                    role="img" aria-label="Inventory" {}
                            }
                            }
                    }
            });
            CharacterPortraitView {
                id: member.id,
                name: &member.name,
                alive: member.alive,
                active: is_active,
                selected: selected_character_id == Some(member.id),
                href: inspection_href,
                title: format!("Inspect {}", member.name),
                aria_label: format!("Inspect {}", member.name),
                decoration: Some(html! {
                    span class="portrait-condition-badge" {
                    span class="incapacitation-wheel"
                        data-strategic-condition-wheel=(member.id)
                        role="img"
                        aria-label="Loading strategic condition"
                        title="Loading strategic condition" tabindex="0" {}
                    }
                }),
                badge: None,
                actions,
            }
        })
        .collect::<Vec<_>>();
    character_portrait_overlay("Active party", inventory, &portraits)
}

/// Leaving belongs to the active character's profile, not the navigation tabs.
pub(crate) fn profile_membership(
    member: &CharacterView,
    party: &[CharacterView],
    location: &str,
) -> Markup {
    html! {
        @if member.alive && party.first().is_some_and(|leader| leader.id != member.id) {
            (crate::templates::sidebar_section("Party", html! {
                form method="post" action=(format!("{location}/party/{}/remove", member.id)) {
                    button type="submit" class="btn btn-danger btn-block" { "Leave party" }
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notified_social_action_stays_visible_while_portrait_keeps_inspection() {
        let member = CharacterView {
            id: 12,
            name: "Greta".into(),
            xp: 0,
            level: 1,
            current_settlement_id: Some("lubeck".into()),
            current_case_site_id: None,
            party_id: Some("party".into()),
            age_years: 24,
            alive: true,
            temporary: false,
            social_notification_count: 2,
            automatic_social_chat_enabled: false,
        };
        let markup = party_portrait_overlay(
            std::slice::from_ref(&member),
            Some(&member),
            "/locations/settlement/lubeck",
            None,
        )
        .into_string();
        assert!(markup.contains(
            "class=\"party-portrait-select\" href=\"/locations/settlement/lubeck/party/12\""
        ));
        assert!(
            markup.contains("class=\"portrait-tab party-social-action party-social-notified\"")
        );
        assert!(markup.contains("href=\"/locations/settlement/lubeck/party/12/social\""));
        assert!(markup.contains("class=\"party-social-notification\""));
        assert!(markup.contains("2 unaddressed morale concerns"));
        assert!(markup.contains("/static/icons/game/conversation.svg"));
        assert!(markup.contains("class=\"incapacitation-wheel\""));
        assert!(markup.contains("data-strategic-condition-wheel=\"12\""));

        let mut quiet = member;
        quiet.social_notification_count = 0;
        let quiet_markup = party_portrait_overlay(
            &[quiet.clone()],
            Some(&quiet),
            "/locations/settlement/lubeck",
            None,
        )
        .into_string();
        assert!(!quiet_markup.contains("party-social-notification"));
        assert!(quiet_markup.contains("class=\"portrait-tab party-social-action\""));
        assert!(quiet_markup.contains("/party/12/social"));
        assert!(quiet_markup.contains("aria-label=\"Open conversation with Greta\""));

        let mut automatic = quiet;
        automatic.social_notification_count = 2;
        automatic.automatic_social_chat_enabled = true;
        let automatic_markup = party_portrait_overlay(
            &[automatic.clone()],
            Some(&automatic),
            "/locations/settlement/lubeck",
            None,
        )
        .into_string();
        assert!(automatic_markup.contains("class=\"party-social-notification\""));
        assert!(automatic_markup.contains("2 unaddressed morale concerns"));
        assert!(!automatic_markup.contains("party-social-notified"));
    }
}
