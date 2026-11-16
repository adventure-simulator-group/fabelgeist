//! Shared authorized character documents, independent of the scene view.

use super::super::side_panels::{PanelKind, PanelSide, PanelSubject, PanelTab, SidePanel};
use super::*;

pub(crate) fn character_sheet_markup(view: CharacterSheetView<'_>) -> Markup {
    let mut left_tabs = vec![PanelTab {
        kind: PanelKind::Attributes,
        content: html! {
            (party_attributes_rail(
                view.attributes_title, view.attributes, view.limbs, view.medical,
                view.physiology_dialog_id, view.surgery, view.injuries, view.projectiles,
            ))
            (view.left_after)
        },
    }];
    if let Some(inventory) = view.inventory {
        left_tabs.push(PanelTab {
            kind: PanelKind::Inventory,
            content: inventory,
        });
    }
    let skills = html! {
        (character_summary_rail(
            view.capability, view.attributes, view.skills, view.combat_profile,
            view.training_religion_id.and_then(OfficialReligion::from_id),
        ))
        (party_skills_rail(
            view.skills_title, view.attributes, view.skills, view.limbs, view.schedule,
            view.schedule_action, view.activity_preview, view.activity_location,
            view.professes_religion, view.prayer_religion_check, view.training_religion_id,
            view.combat_profile, view.skill_actions,
        ))
        (view.right_after)
    };
    let identity = html! {
        (character_bio_rail(
            view.character, view.religion_id, view.fame, view.infamy, view.personality,
            view.can_renounce, view.location_path, view.organization_memberships,
            view.organization_presentation, view.organization_minute,
        ))
    };
    html! {
        aside class="left-sidebar" {
            (SidePanel {
                side: PanelSide::Left,
                subject: PanelSubject::Character(view.character),
                tabs: left_tabs,
            }.render())
        }
        main class="center-content settlement-main party-member-stage" {
            (view.center_before)
            (view.portraits)
            (visual_stage(VisualStageKind::Character, &view.character.name, view.description))
            (view.center_after)
        }
        aside class="right-sidebar" {
            (SidePanel {
                side: PanelSide::Right,
                subject: PanelSubject::Character(view.character),
                tabs: vec![
                    PanelTab { kind: PanelKind::Skills, content: skills },
                    PanelTab { kind: PanelKind::Character, content: identity },
                ],
            }.render())
        }
        (view.after)
    }
}

/// Modal documents share the character sheet lifecycle and remain outside tabs.
pub(super) fn personal_dialogs(
    medical: &MedicalPresentation,
    name: &str,
    foraging: Option<Markup>,
) -> Markup {
    html! {
        (physiology_dialog(medical, "physiology-chart-dialog", name))
        @if let Some(dialog) = foraging { (dialog) }
    }
}

pub(super) fn membership_actions(
    location_path: &str,
    active_character: &CharacterView,
    selected: &CharacterView,
    active_party: Option<&PartyView>,
    selected_party: Option<&PartyView>,
) -> Markup {
    html! {
        @if selected.id != active_character.id {
                @if active_character.party_id == selected.party_id {
                    @if active_party.is_some_and(|party| party.leader_id == selected.id) {
                        (sidebar_section("Party", html! {
                            form method="post" action=(format!("{location_path}/party/{}/remove", active_character.id)) {
                                button type="submit" class="btn btn-danger btn-block" { "Leave party" }
                            }
                        }))
                    } @else {
                        (sidebar_section("Party", html! {
                            form method="post" action=(format!("{location_path}/party/{}/remove", selected.id)) {
                                button type="submit" class="btn btn-danger btn-block" {
                                    @if active_party.is_some_and(|party| party.leader_id == active_character.id) { "Kick from party" }
                                    @else { "Request kick" }
                                }
                            }
                        }))
                    }
                } @else if let Some(party) = selected_party {
                    (sidebar_section("Party", html! {
                        p { (&party.name) }
                        form method="post" action=(format!("/parties/{}/join-general", party.id)) {
                            button type="submit" class="btn btn-primary btn-block" { "Request to join party" }
                        }
                    }))
                }
            }
    }
}
