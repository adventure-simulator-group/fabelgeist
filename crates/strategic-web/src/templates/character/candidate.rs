//! Fallible starter projection at the character-preview boundary.

use super::super::settlement::CharacterSkillHours;
use crate::spacetimedb::{
    BackendOrganizationMembership, CharacterAttributes, CharacterCapability, CharacterLimbs,
    CharacterSkills, CharacterView, Conscience, Conviction, Courtship, Drive, Hygiene, Mirth,
    Nerve, OrganizationMembershipStatus, OrganizationPresentation, Outlook, Personality,
    SelfKnowledge, SelfRegard, Sociability, Temperance, Transparency,
};
use adventuresim_core::equipment::LoadoutSlot;
use adventuresim_core::starting_character::{StartingCharacterSpec, StartingPersonalityTrait};
use adventuresim_core::{
    equipment::weapon_skill_distribution_for_item,
    skill::{PlayerSkills, Skill},
    strategic_schedule::{CombatTrainingProfile, EquippedCombatItem},
};

pub(super) struct CandidatePresentation {
    pub(super) character: CharacterView,
    pub(super) attributes: CharacterAttributes,
    pub(super) capability: CharacterCapability,
    pub(super) limbs: CharacterLimbs,
    pub(super) personality: Personality,
    pub(super) skills: CharacterSkills,
    pub(super) religion_id: Option<String>,
    pub(super) organization_memberships: Vec<BackendOrganizationMembership>,
    pub(super) organization_presentation: Option<OrganizationPresentation>,
    pub(super) combat_profile: CombatTrainingProfile,
}

impl TryFrom<&StartingCharacterSpec> for CandidatePresentation {
    type Error = adventuresim_world_schema::person_names::NameCatalogError;

    fn try_from(spec: &StartingCharacterSpec) -> Result<Self, Self::Error> {
        let character = CharacterView {
            id: spec.id,
            name: spec.native_everyday_name()?.into_string(),
            xp: 0,
            level: 1,
            current_settlement_id: None,
            current_case_site_id: None,
            party_id: None,
            age_years: spec.age_years,
            alive: true,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        };
        let attributes = candidate_attributes(spec);
        let skills = candidate_skills(spec);
        let combat_profile = candidate_combat_profile(spec);
        let capability = candidate_capability(spec, &skills);
        let organization_memberships = candidate_memberships(spec);
        let organization_presentation =
            spec.organization
                .as_ref()
                .map(|organization| OrganizationPresentation {
                    character_id: spec.id,
                    organization_id: organization.organization_id.clone(),
                });
        Ok(Self {
            character,
            attributes,
            capability,
            limbs: CharacterLimbs {
                character_id: spec.id,
                left_arm_health: 1.0,
                right_arm_health: 1.0,
                left_leg_health: 1.0,
                right_leg_health: 1.0,
                head_health: 1.0,
                chest_health: 1.0,
                stomach_health: 1.0,
            },
            personality: candidate_personality(spec),
            skills,
            religion_id: spec.religion_id.clone(),
            organization_memberships,
            organization_presentation,
            combat_profile,
        })
    }
}

fn candidate_personality(spec: &StartingCharacterSpec) -> Personality {
    let mut personality = Personality {
        nerve: Nerve::Neutral,
        drive: Drive::Neutral,
        outlook: Outlook::Neutral,
        sociability: Sociability::Neutral,
        conscience: Conscience::Neutral,
        self_regard: SelfRegard::Neutral,
        conviction: Conviction::Neutral,
        hygiene: Hygiene::Neutral,
        temperance: Temperance::Neutral,
        mirth: Mirth::Neutral,
        courtship: Courtship::Neutral,
        transparency: Transparency::Neutral,
        self_knowledge: SelfKnowledge::Neutral,
        sex: spec.personality.sex,
        presentation: spec.personality.presentation,
        inclination: spec.personality.inclination,
    };
    for personality_trait in &spec.personality.traits {
        match personality_trait {
            StartingPersonalityTrait::Brave => personality.nerve = Nerve::Brave,
            StartingPersonalityTrait::Fearful => personality.nerve = Nerve::Fearful,
            StartingPersonalityTrait::Ambitious => personality.drive = Drive::Ambitious,
            StartingPersonalityTrait::Content => personality.drive = Drive::Content,
            StartingPersonalityTrait::Sanguine => personality.outlook = Outlook::Sanguine,
            StartingPersonalityTrait::Brooding => personality.outlook = Outlook::Brooding,
            StartingPersonalityTrait::Gregarious => {
                personality.sociability = Sociability::Gregarious
            }
            StartingPersonalityTrait::Solitary => personality.sociability = Sociability::Solitary,
            StartingPersonalityTrait::Compassionate => {
                personality.conscience = Conscience::Compassionate
            }
            StartingPersonalityTrait::Callous => personality.conscience = Conscience::Callous,
            StartingPersonalityTrait::Cruel => personality.conscience = Conscience::Cruel,
            StartingPersonalityTrait::Proud => personality.self_regard = SelfRegard::Proud,
            StartingPersonalityTrait::Humble => personality.self_regard = SelfRegard::Humble,
            StartingPersonalityTrait::Zealous => personality.conviction = Conviction::Zealous,
            StartingPersonalityTrait::Irreverent => personality.conviction = Conviction::Irreverent,
            StartingPersonalityTrait::Slovenly => personality.hygiene = Hygiene::Slovenly,
            StartingPersonalityTrait::Cleanly => personality.hygiene = Hygiene::Cleanly,
            StartingPersonalityTrait::Temperate => personality.temperance = Temperance::Temperate,
            StartingPersonalityTrait::Drunkard => personality.temperance = Temperance::Drunkard,
            StartingPersonalityTrait::Merry => personality.mirth = Mirth::Merry,
            StartingPersonalityTrait::Grave => personality.mirth = Mirth::Grave,
            StartingPersonalityTrait::Amorous => personality.courtship = Courtship::Amorous,
            StartingPersonalityTrait::Proper => personality.courtship = Courtship::Proper,
            StartingPersonalityTrait::Open => personality.transparency = Transparency::Open,
            StartingPersonalityTrait::Guarded => personality.transparency = Transparency::Guarded,
            StartingPersonalityTrait::Introspective => {
                personality.self_knowledge = SelfKnowledge::Introspective
            }
            StartingPersonalityTrait::SelfDeceiving => {
                personality.self_knowledge = SelfKnowledge::SelfDeceiving
            }
        }
    }
    personality
}

fn candidate_attributes(spec: &StartingCharacterSpec) -> CharacterAttributes {
    CharacterAttributes {
        character_id: spec.id,
        endurance: spec.attributes.endurance,
        immunity: spec.attributes.immunity,
        gut: spec.attributes.gut,
        intelligence: spec.attributes.intelligence,
        instinct: spec.attributes.instinct,
        eyesight: spec.attributes.eyesight,
        hearing: spec.attributes.hearing,
        left_arm_strength: spec.attributes.strength,
        right_arm_strength: spec.attributes.strength,
        left_leg_strength: spec.attributes.strength,
        right_leg_strength: spec.attributes.strength,
        left_arm_agility: spec.attributes.agility,
        right_arm_agility: spec.attributes.agility,
        left_leg_agility: spec.attributes.agility,
        right_leg_agility: spec.attributes.agility,
    }
}

fn candidate_skills(spec: &StartingCharacterSpec) -> CharacterSkills {
    CharacterSkills {
        character_id: spec.id,
        polearm_hours: spec.skills.polearm,
        axe_hours: spec.skills.axe,
        bludgeon_hours: spec.skills.bludgeon,
        sword_hours: spec.skills.sword,
        knife_hours: spec.skills.knife,
        dodge_hours: spec.skills.dodge,
        block_hours: spec.skills.block,
        bow_hours: spec.skills.bow,
        crossbow_hours: spec.skills.crossbow,
        firearm_hours: spec.skills.firearm,
        throw_hours: spec.skills.throw,
        will_hours: spec.skills.will,
        insight_hours: spec.skills.insight,
        charm_hours: spec.skills.charm,
        command_hours: spec.skills.command,
        deception_hours: spec.skills.deception,
        physiology_hours: spec.skills.physiology,
        cooking_hours: spec.skills.cooking,
        herbalism_hours: spec.skills.herbalism,
        religion_hours: crate::spacetimedb::religion_hours_from_core(&spec.skills.religion),
        oral_languages: crate::spacetimedb::empty_oral_language_hours(),
        written_languages: crate::spacetimedb::empty_written_language_hours(),
        stealth_hours: spec.skills.stealth,
        balance_hours: spec.skills.balance,
        terrain_plains_hours: spec.skills.terrain_plains,
        terrain_forest_hours: spec.skills.terrain_forest,
        terrain_hills_hours: spec.skills.terrain_hills,
        terrain_wetlands_hours: spec.skills.terrain_wetlands,
        terrain_urban_hours: spec.skills.terrain_urban,
        terrain_snow_hours: spec.skills.terrain_snow,
        bestiary_hours: crate::spacetimedb::bestiary_hours_from_core(&spec.skills.bestiary),
        surgery_hours: spec.skills.surgery,
        tailoring_hours: spec.skills.tailoring,
        smithing_hours: spec.skills.smithing,
    }
}

fn candidate_combat_profile(spec: &StartingCharacterSpec) -> CombatTrainingProfile {
    CombatTrainingProfile::from_equipped_hands(
        spec.inventory
            .iter()
            .filter(|item| {
                matches!(
                    item.equipped,
                    Some(LoadoutSlot::LeftHand | LoadoutSlot::RightHand)
                )
            })
            .map(|item| {
                let shield = matches!(
                    item.item_id.as_str(),
                    "buckler" | "targe" | "heater_shield" | "round_shield" | "pavise"
                );
                EquippedCombatItem {
                    weapons: if shield {
                        Default::default()
                    } else {
                        weapon_skill_distribution_for_item(&item.item_id)
                    },
                    shield,
                    balance: 1.0,
                }
            }),
    )
}

fn candidate_weapon_roles(equipped_item_ids: &[&str]) -> (bool, bool) {
    let has = |choices: &[&str]| equipped_item_ids.iter().any(|item| choices.contains(item));
    let ranged = has(&[
        "self_bow",
        "longbow",
        "light_crossbow",
        "heavy_crossbow",
        "matchlock_arquebus",
        "hooked_arquebus",
    ]);
    let melee = has(&[
        "arming_sword",
        "baselard",
        "bauernwehr",
        "club",
        "flanged_mace",
        "halberd",
        "hand_axe",
        "hunting_spear",
        "katzbalger",
        "kriegsmesser",
        "longsword",
        "messer",
        "military_pike",
        "misericorde",
        "rapier",
        "rondel_dagger",
        "utility_knife",
        "walking_staff",
        "war_hammer",
        "zweihander",
    ]);
    (melee, ranged)
}

fn candidate_capability(
    spec: &StartingCharacterSpec,
    skills: &CharacterSkills,
) -> CharacterCapability {
    let effective_skill_hours = CharacterSkillHours(skills);
    let armor_slots = spec
        .inventory
        .iter()
        .filter(|item| {
            matches!(
                item.equipped,
                Some(
                    LoadoutSlot::LeftArm
                        | LoadoutSlot::RightArm
                        | LoadoutSlot::LeftLeg
                        | LoadoutSlot::RightLeg
                        | LoadoutSlot::Head
                        | LoadoutSlot::Chest
                        | LoadoutSlot::Stomach
                )
            )
        })
        .count();
    let equipped_item_ids = spec
        .inventory
        .iter()
        .filter(|item| item.equipped.is_some())
        .map(|item| item.item_id.as_str())
        .collect::<Vec<_>>();
    let has = |choices: &[&str]| equipped_item_ids.iter().any(|item| choices.contains(item));
    let (melee, ranged) = candidate_weapon_roles(&equipped_item_ids);
    let weapon_precision = equipped_item_ids
        .iter()
        .filter_map(|id| adventuresim_core::item_catalog::weapon_precision(id))
        .fold(0.0_f32, f32::max);
    CharacterCapability {
        character_id: spec.id,
        melee,
        ranged,
        heavy: has(&[
            "heavy_crossbow",
            "hooked_arquebus",
            "military_pike",
            "war_hammer",
            "zweihander",
        ]),
        quarter_armor: armor_slots >= 2,
        half_armor: armor_slots >= 4,
        three_quarter_armor: armor_slots >= 6,
        full_armor: armor_slots >= 7,
        athletics: Skill::Dodge
            .capped_training_rank(spec.skills.dodge, &spec.attributes)
            .max(Skill::Balance.capped_training_rank(spec.skills.balance, &spec.attributes)),
        endurance: spec.attributes.endurance,
        physiology: Skill::Physiology
            .capped_training_rank(spec.skills.physiology, &spec.attributes),
        knife: Skill::Knife.capped_training_rank(spec.skills.knife, &spec.attributes),
        tailoring: Skill::Tailoring.capped_training_rank(spec.skills.tailoring, &spec.attributes),
        surgery: Skill::Surgery.capped_training_rank(
            effective_skill_hours.effective_skill_hours(Skill::Surgery),
            &spec.attributes,
        ),
        command: Skill::Command.capped_training_rank(spec.skills.command, &spec.attributes),
        religion: Skill::Religion
            .capped_training_rank(spec.skills.religion.maximum_effective(), &spec.attributes),
        weapon_precision,
        autoresolve_combat_power: 0,
    }
}

fn candidate_memberships(spec: &StartingCharacterSpec) -> Vec<BackendOrganizationMembership> {
    spec.organization
        .iter()
        .map(|organization| {
            let paid_through =
                adventuresim_core::organization::organization(&organization.organization_id)
                    .and_then(|definition| definition.dues.as_ref())
                    .map_or(
                        adventuresim_world_schema::calendar::StrategicMinute::MAX,
                        |dues| {
                            adventuresim_world_schema::calendar::StrategicMinute::ZERO
                                .saturating_add_days(u64::from(dues.interval_days))
                        },
                    );
            BackendOrganizationMembership {
                id: 0,
                character_id: spec.id,
                organization_id: organization.organization_id.clone(),
                role_id: organization.role_id.clone(),
                joined_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
                dues_paid_through_minute: adventuresim_stdb_client::StrategicMinute {
                    minutes: paid_through.get(),
                },
                status: OrganizationMembershipStatus::Active,
                apprenticeship_minutes_accrued: 0,
                practice_minutes_accrued: 0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::personality::{Inclination, Presentation};

    #[test]
    fn candidate_view_keeps_canonical_personality_fields() {
        for presentation in [
            Presentation::Man,
            Presentation::Ambiguous,
            Presentation::Woman,
        ] {
            for inclination in [
                Inclination::Men,
                Inclination::Either,
                Inclination::Women,
                Inclination::Neither,
            ] {
                let mut spec = adventuresim_core::starting_character::default_character(
                    "canonical-personality-fixture",
                );
                spec.personality.presentation = presentation;
                spec.personality.inclination = inclination;
                let view = CandidatePresentation::try_from(&spec).unwrap();
                assert_eq!(view.personality.presentation, presentation);
                assert_eq!(view.personality.inclination, inclination);
            }
        }
    }
}
