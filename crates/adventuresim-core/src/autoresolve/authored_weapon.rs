//! Checked authored melee-weapon projection shared by threats and iteration builds.

use super::CombatWeapon;
use crate::combat::EMBEDDED_AUTORESOLVE_PARAMETERS;
use crate::elapsed::ElapsedError;
use crate::item_catalog_schema::ItemKind;
use adventuresim_weapon_model::ValidationError;

pub type AuthoredMeleeWeaponResult<T> = Result<T, AuthoredMeleeWeaponError>;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AuthoredMeleeWeaponError {
    #[error("{weapon} is not a weapon")]
    NotWeapon { weapon: String },
    #[error("{weapon} is not melee-only")]
    NotMeleeOnly { weapon: String },
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error("{weapon} has no generated recipe")]
    MissingRecipe { weapon: String },
    #[error("{weapon}: {violations:?}")]
    Recipe {
        weapon: String,
        violations: Vec<ValidationError>,
    },
}

impl CombatWeapon {
    pub(super) fn from_authored_melee_definition(
        definition: &crate::item_catalog::ItemDefinition,
    ) -> AuthoredMeleeWeaponResult<Self> {
        let weapon_id = definition.id.as_str();
        let ItemKind::Weapon {
            preferred_attack,
            reach_m,
            precision,
            melee,
            ranged,
            skills,
            moment_of_inertia_kg_m2,
            ..
        } = &definition.kind
        else {
            return Err(AuthoredMeleeWeaponError::NotWeapon {
                weapon: weapon_id.to_owned(),
            });
        };
        if !melee || *ranged {
            return Err(AuthoredMeleeWeaponError::NotMeleeOnly {
                weapon: weapon_id.to_owned(),
            });
        }
        let moment_of_inertia_kg_m2 = *moment_of_inertia_kg_m2;
        let timing = crate::equipment::melee_attack_timing(
            *preferred_attack,
            moment_of_inertia_kg_m2,
            false,
        )?;
        let grip_to_tip_m = definition
            .equipment
            .as_ref()
            .map_or(0.0, |equipment| equipment.physical.grip_to_tip_m);
        let dimensions_m = definition
            .equipment
            .as_ref()
            .map_or([0.0; 3], |equipment| equipment.physical.dimensions_m);
        let total_length_m = dimensions_m[1];
        let recipe = adventuresim_weapon_model::default_design(weapon_id)
            .ok_or_else(|| AuthoredMeleeWeaponError::MissingRecipe {
                weapon: weapon_id.to_owned(),
            })?;
        let striking_head_length_m = adventuresim_weapon_model::derive_properties(&recipe)
            .map_err(|violations| AuthoredMeleeWeaponError::Recipe {
                weapon: weapon_id.to_owned(),
                violations,
            })?
            .striking_head_length_m;
        let skill_distribution: crate::equipment::WeaponSkillDistribution = (*skills).into();
        Ok(Self {
            skills: skill_distribution,
            melee: true,
            ranged: false,
            preferred_melee_style: *preferred_attack,
            weight: definition.weight_kg,
            moment_of_inertia_kg_m2,
            precision: *precision,
            melee_reach: *reach_m,
            grip_to_tip_m,
            total_length_m,
            striking_head_length_m,
            distal_headed: crate::combat::has_distal_striking_surface(
                grip_to_tip_m,
                striking_head_length_m,
                definition
                    .equipment
                    .as_ref()
                    .and_then(|equipment| equipment.material),
                definition
                    .equipment
                    .as_ref()
                    .and_then(|equipment| equipment.striking_material),
            ),
            body_material: definition
                .equipment
                .as_ref()
                .and_then(|equipment| equipment.material),
            striking_material: definition
                .equipment
                .as_ref()
                .and_then(|equipment| equipment.striking_material),
            ranged_range: 0.0,
            // Autoresolve advances the same complete attack cycle as the tactical
            // bot: the authored server windup to contact plus inertia/style-based
            // recovery before another fresh attack may begin.
            attack_interval_seconds: EMBEDDED_AUTORESOLVE_PARAMETERS
                .melee_windup_seconds
                .checked_add(timing.recovery_secs)?,
            balance: crate::equipment::weapon_balance_from_moment(
                moment_of_inertia_kg_m2,
                definition.weight_kg,
                grip_to_tip_m,
            ),
            ranged_force_joules: 0.0,
        })
    }
}
