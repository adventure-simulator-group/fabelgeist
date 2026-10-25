"""Apply explicit source-text replacements; no Rust parsing or project execution."""
from pathlib import Path
import hashlib
import json

reports = Path(__file__).resolve().parent
worktree = reports.parent / 'rig-review-774/worktree'
changes = {}
proof = []

def read(name):
    if name not in changes:
        changes[name] = (worktree / name).read_text()
    return changes[name]

def replace(name, before, after):
    source = read(name)
    assert source.count(before) == 1, (name, before, source.count(before))
    changes[name] = source.replace(before, after, 1)
    proof.append({'path': name, 'before': before, 'after': after})

roster = 'crates/adventuresim-core/src/autoresolve/melee_iteration.rs'
s = read(roster)
a = s.index('pub(super) fn authored_melee_weapon(')
b = s.index('fn apply_authored_armor(', a)
old_weapon = s[a:b]
weapon = old_weapon
weapon = weapon.replace(') -> Result<CombatWeapon, String> {', ') -> AuthoredMeleeWeaponResult<CombatWeapon> {')
weapon = weapon.replace('        skills,\n        ..', '        skills,\n        moment_of_inertia_kg_m2,\n        ..', 1)
weapon = weapon.replace('return Err(format!("{weapon_id} is not a weapon"));', 'return Err(AuthoredMeleeWeaponError::NotWeapon {\n            weapon: weapon_id.to_owned(),\n        });')
weapon = weapon.replace('return Err(format!("{weapon_id} is not melee-only"));', 'return Err(AuthoredMeleeWeaponError::NotMeleeOnly {\n            weapon: weapon_id.to_owned(),\n        });')
start = weapon.index('    let moment_of_inertia_kg_m2 = match definition.kind {')
end = weapon.index('    let timing =', start)
weapon = weapon[:start] + '    let moment_of_inertia_kg_m2 = *moment_of_inertia_kg_m2;\n' + weapon[end:]
weapon = weapon.replace('crate::equipment::melee_attack_timing(*preferred_attack, moment_of_inertia_kg_m2, false)\n            .map_err(|error| error.to_string())?', 'crate::equipment::melee_attack_timing(*preferred_attack, moment_of_inertia_kg_m2, false)?')
weapon = weapon.replace('.ok_or_else(|| format!("{weapon_id} has no generated recipe"))?', '.ok_or_else(|| AuthoredMeleeWeaponError::MissingRecipe {\n            weapon: weapon_id.to_owned(),\n        })?')
weapon = weapon.replace('.map_err(|errors| format!("{weapon_id}: {errors:?}"))?', '.map_err(|violations| AuthoredMeleeWeaponError::Recipe {\n            weapon: weapon_id.to_owned(),\n            violations,\n        })?')
weapon = weapon.replace('.checked_add(timing.recovery_secs)\n            .map_err(|error| error.to_string())?', '.checked_add(timing.recovery_secs)?')
assert 'map_err(|error| error.to_string())' not in weapon
assert 'unreachable!' not in weapon
header = '''//! Checked authored melee-weapon projection shared by threats and iteration builds.

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

'''
new_owner = 'crates/adventuresim-core/src/autoresolve/authored_weapon.rs'
assert not (worktree / new_owner).exists()
changes[new_owner] = header + weapon.rstrip() + '\n'
replace(roster, old_weapon, '')
replace(roster, '//! Melee-only iteration roster assembled from canonical character and item data.\n\nuse super::*;\n', '//! Melee-only iteration roster assembled from canonical character and item data.\n\nuse super::*;\nuse super::authored_weapon::authored_melee_weapon;\n')
replace(roster, 'mod evidence;\n\npub use evidence::*;', 'pub use error::{MeleeIterationError, MeleeIterationResult};\npub use evidence::*;\n\nmod error;\nmod evidence;')
replace(roster, 'pub fn melee_iteration_roster() -> Result<(MeleeIterationBuild, Vec<MeleeIterationBuild>), String>', 'pub fn melee_iteration_roster() -> MeleeIterationResult<(MeleeIterationBuild, Vec<MeleeIterationBuild>)>')
replace(roster, '        .native_everyday_name()\n        .map_err(|error| error.to_string())?', '        .native_everyday_name()?')
assert read(roster).count(') -> Result<MeleeIterationBuild, String>') == 2
changes[roster] = read(roster).replace(') -> Result<MeleeIterationBuild, String>', ') -> MeleeIterationResult<MeleeIterationBuild>')
replace(roster, ') -> Result<CombatEquipment, String>', ') -> MeleeIterationResult<CombatEquipment>')
replace(roster, '.ok_or_else(|| format!("unknown weapon {weapon_id}"))?', '.ok_or_else(|| MeleeIterationError::UnknownWeapon { weapon: weapon_id.to_owned() })?')
replace(roster, '.ok_or_else(|| format!("unknown shield {shield_id}"))?', '.ok_or_else(|| MeleeIterationError::UnknownShield { shield: shield_id.to_owned() })?')
replace(roster, 'return Err(format!("{shield_id} is not a shield"));', 'return Err(MeleeIterationError::NotShield { shield: shield_id.to_owned() });')
replace(roster, 'fn apply_authored_armor(equipment: &mut CombatEquipment, armor_ids: &[&str]) -> Result<(), String>', 'fn apply_authored_armor(equipment: &mut CombatEquipment, armor_ids: &[&str]) -> MeleeIterationResult<()>')
replace(roster, '.ok_or_else(|| format!("unknown armor {armor_id}"))?', '.ok_or_else(|| MeleeIterationError::UnknownArmor { armor: (*armor_id).to_owned() })?')
replace(roster, '.ok_or_else(|| format!("armor {armor_id} has no equipment projection"))?', '.ok_or_else(|| MeleeIterationError::ArmorProjectionUnavailable { armor: (*armor_id).to_owned() })?')
replace(roster, '.ok_or_else(|| format!("armor {armor_id} has no authored placement"))?', '.ok_or_else(|| MeleeIterationError::ArmorPlacementUnavailable { armor: (*armor_id).to_owned() })?')
replace(roster, 'return Err(format!("armor {armor_id} placement protects no body part"));', 'return Err(MeleeIterationError::EmptyArmorProtection { armor: (*armor_id).to_owned() });')
parent = 'crates/adventuresim-core/src/autoresolve.rs'
replace(parent, 'pub use classification::{', 'pub use authored_weapon::{AuthoredMeleeWeaponError, AuthoredMeleeWeaponResult};\npub use classification::{')
replace(parent, 'mod classification;', 'mod authored_weapon;\nmod classification;')
threat = 'crates/adventuresim-core/src/autoresolve/threat.rs'
replace(threat, 'use super::*;\n', 'use super::*;\nuse super::authored_weapon::authored_melee_weapon;\n')
replace(threat, '    Timing(#[from] ElapsedError),', '    Timing(#[from] ElapsedError),\n    #[error("authored threat sidearm knife is absent from the item catalogue")]\n    MissingSidearmDefinition,\n    #[error(transparent)]\n    Sidearm(#[from] AuthoredMeleeWeaponError),')
replace(threat, '    equip_threat_weapon(&mut combatant, weapon, profile.ranged);', '    equip_threat_weapon(&mut combatant, weapon, profile.ranged)?;')
replace(threat, 'fn equip_threat_weapon(combatant: &mut Combatant, weapon: CombatWeapon, ranged: bool) {', 'fn equip_threat_weapon(\n    combatant: &mut Combatant,\n    weapon: CombatWeapon,\n    ranged: bool,\n) -> AuthoredThreatResult<()> {')
replace(threat, '''    if ranged {
        combatant.equipment.ranged_weapon = Some(weapon);
        combatant.equipment.ranged_projectile_kind = Some(ProjectileKind::Arrowhead);
        combatant.equipment.melee_weapon = Some(
            super::melee_iteration::authored_melee_weapon(
                crate::item_catalog::definition("knife").expect("catalog knife"),
            )
            .expect("valid generated sidearm"),
        );''', '''    if ranged {
        let sidearm = authored_melee_weapon(
            crate::item_catalog::definition("knife")
                .ok_or(AuthoredThreatError::MissingSidearmDefinition)?,
        )?;
        combatant.equipment.ranged_weapon = Some(weapon);
        combatant.equipment.ranged_projectile_kind = Some(ProjectileKind::Arrowhead);
        combatant.equipment.melee_weapon = Some(sidearm);''')
replace(threat, '        combatant.equipment.melee_weapon = Some(weapon);\n    }\n}', '        combatant.equipment.melee_weapon = Some(weapon);\n    }\n    Ok(())\n}')
error = 'crates/adventuresim-core/src/autoresolve/melee_iteration/error.rs'
assert not (worktree / error).exists()
changes[error] = '''//! Failures from building the authored melee roster and its acceptance evidence.

use super::super::{AuthoredMeleeWeaponError, BattleResolutionError};
use crate::elapsed::ElapsedError;
use crate::unit_interval::FractionError;
use adventuresim_world_schema::person_names::NameCatalogError;

pub type MeleeIterationResult<T> = Result<T, MeleeIterationError>;

#[derive(Debug, thiserror::Error)]
pub enum MeleeIterationError {
    #[error(transparent)]
    Name(#[from] NameCatalogError),
    #[error(transparent)]
    Weapon(#[from] AuthoredMeleeWeaponError),
    #[error(transparent)]
    Resolution(#[from] BattleResolutionError),
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error(transparent)]
    Performance(#[from] FractionError),
    #[error("unknown weapon {weapon}")]
    UnknownWeapon { weapon: String },
    #[error("unknown shield {shield}")]
    UnknownShield { shield: String },
    #[error("{shield} is not a shield")]
    NotShield { shield: String },
    #[error("unknown armor {armor}")]
    UnknownArmor { armor: String },
    #[error("armor {armor} has no equipment projection")]
    ArmorProjectionUnavailable { armor: String },
    #[error("armor {armor} has no authored placement")]
    ArmorPlacementUnavailable { armor: String },
    #[error("armor {armor} placement protects no body part")]
    EmptyArmorProtection { armor: String },
    #[error("missing {key}")]
    MissingOpponent { key: &'static str },
    #[error("missing {item}")]
    MissingEvidenceItem { item: &'static str },
    #[error("{item} has no equipment metadata")]
    MissingEvidenceEquipment { item: &'static str },
    #[error("John has no melee weapon")]
    MissingEvidenceMeleeWeapon,
    #[error("bounded hammer seeds produced no cancellation evidence")]
    CancellationUnavailable,
}
'''
evidence = 'crates/adventuresim-core/src/autoresolve/melee_iteration/evidence.rs'
replace(evidence, 'pub fn melee_iteration_acceptance_evidence() -> Result<MeleeIterationAcceptanceEvidence, String>', 'pub fn melee_iteration_acceptance_evidence() -> MeleeIterationResult<MeleeIterationAcceptanceEvidence>')
replace(evidence, '.ok_or_else(|| format!("missing {key}"))', '.ok_or(MeleeIterationError::MissingOpponent { key })')
replace(evidence, 'fn fatigue_cadence_evidence(build: &MeleeIterationBuild) -> Result<FatigueCadenceEvidence, String>', 'fn fatigue_cadence_evidence(build: &MeleeIterationBuild) -> MeleeIterationResult<FatigueCadenceEvidence>')
replace(evidence, '.ok_or("John has no melee weapon")?', '.ok_or(MeleeIterationError::MissingEvidenceMeleeWeapon)?')
assert read(evidence).count('.map_err(|error| error.to_string())?') == 7
changes[evidence] = read(evidence).replace('.map_err(|error| error.to_string())?', '?')
armor = 'crates/adventuresim-core/src/autoresolve/melee_iteration/evidence/armor.rs'
replace(armor, ') -> Result<Vec<ForcedArmorContactEvidence>, String>', ') -> MeleeIterationResult<Vec<ForcedArmorContactEvidence>>')
replace(armor, 'pub(super) fn mirrored_vambrace_contacts() -> Result<Vec<MirroredArmorContactEvidence>, String>', 'pub(super) fn mirrored_vambrace_contacts() -> MeleeIterationResult<Vec<MirroredArmorContactEvidence>>')
replace(armor, '.ok_or("missing authored brigandine")?', '.ok_or(MeleeIterationError::MissingEvidenceItem { item: "brigandine" })?')
replace(armor, '.ok_or("missing vambrace")?', '.ok_or(MeleeIterationError::MissingEvidenceItem { item: "vambrace" })?')
replace(armor, '.ok_or("brigandine has no equipment metadata")?', '.ok_or(MeleeIterationError::MissingEvidenceEquipment { item: "brigandine" })?')
replace(armor, '.ok_or("vambrace has no equipment metadata")?', '.ok_or(MeleeIterationError::MissingEvidenceEquipment { item: "vambrace" })?')
timeline = 'crates/adventuresim-core/src/autoresolve/melee_iteration/evidence/timeline.rs'
replace(timeline, ') -> Result<AutoresolveTimelineEvidence, String>', ') -> MeleeIterationResult<AutoresolveTimelineEvidence>')
assert read(timeline).count('.map_err(|error| error.to_string())?') == 5
changes[timeline] = read(timeline).replace('.map_err(|error| error.to_string())?', '?')
replace(timeline, 'cancellation.ok_or("bounded hammer seeds produced no cancellation evidence")?', 'cancellation.ok_or(MeleeIterationError::CancellationUnavailable)?')
native = 'crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/error.rs'
replace(native, 'use adventuresim_core::autoresolve::BattleResolutionError;', 'use adventuresim_core::autoresolve::{BattleResolutionError, MeleeIterationError};')
replace(native, '    Resolution(#[from] BattleResolutionError),', '    Resolution(#[from] BattleResolutionError),\n    #[error(transparent)]\n    Evidence(#[from] MeleeIterationError),')
for name, content in changes.items():
    path = worktree / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
(reports / 'P02-authored-error-source-replacements.json').write_text(json.dumps({'changes':proof,'moved_weapon_before':old_weapon,'moved_weapon_after':weapon,'files':list(changes),'new_owner_sha256':hashlib.sha256(changes[new_owner].encode()).hexdigest()},indent=2)+'\n')
print(json.dumps({'changed_files':len(changes),'explicit_unique_replacements':len(proof),'new_owners':[new_owner,error]}))
