"""Verify unarmed source-bound traces and historical qualifications only."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-UNARMED-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def preserved(item):
    p = (root / item['preserved_bytes']).resolve()
    assert p.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(p.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert oid == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 27
core = 'crates/adventuresim-core/src/'
tactical = 'crates/adventuresim-tactical-server/src/combat/'
tcore = 'crates/adventuresim-tactical-core/'
module = 'crates/adventuresim-stdb-module/src/'
must(sources['AGENTS.md'], ['Fatigue is one general value',
 'Weapon work, defenses, dodges, and strenuous movement all contribute to it.',
 'Combat performance depends on total incapacitation, not fatigue directly.',
 'Limb-specific injury penalties remain intentional'])
must(sources[tcore + 'AGENTS.md'], ['content/tactical/combat.yaml',
 'as a typed field', 'pass a typed'])
must(sources[tactical + 'AGENTS.md'], ['content/tactical/combat.yaml', 'typed schema'])
must(sources['content/tactical/combat.yaml'], ['mechanical_efficiency:',
 'capacity_joules_per_endurance:'])
auto = sources[core + 'autoresolve.rs']
must(auto, ['self.equipment.ranged_weapon.is_some() && self.equipment.ammunition > 0',
 'self.equipment.melee_weapon.is_some()',
 'melee_attack_capability(&self.body, &self.equipment.for_melee()).is_available()',
 'self.is_incapacitated() || self.yielded',
 'side.is_empty() || side.iter().all(Combatant::is_defeated)',
 'classify_battle_resolution(side_defeated(&allies), side_defeated(&enemies), false)',
 'yielded: combatant.yielded,', 'incapacitated,'])
preferred = auto.split('fn preferred_attack_mode(', 1)[1].split('fn ranged_exchange(', 1)[0]
must(preferred, ['attacker.can_attack_ranged()', 'BattleAttackKind::Ranged', 'BattleAttackKind::Melee'])
model = sources[core + 'autoresolve/model.rs']
must(model, ['equipment.weapon = self.melee_weapon;', 'self.weapon.map(|_| self.holding_side)',
 'self.weapon.is_none()', 'self.weapon.map_or(0.0, |weapon| weapon.weight)'])
scheduling = sources[core + 'autoresolve/melee_round/scheduling.rs']
must(scheduling, ['preferred_attack_mode(&attackers[index]) != BattleAttackKind::Melee',
 'if !attackers[index].can_attack_melee() {', 'attackers[index].yielded = true;'])
must(sources[core + 'autoresolve/joint_melee.rs'], ['resolve_joint_melee_window(',
 'schedule_both_sides(', 'schedule_side_melee_attacks_in_window('])
must(sources[core + 'combat/capability.rs'], ['let Some(side) = equipment.weapon_holding_side() else {',
 'return MeleeAttackCapability::NoStrikingSide;', 'body.body_part_health(*arm) <= 0.0'])
must(sources[core + 'combat/contact_geometry.rs'], ['if facts.grip_to_tip_metres <= 0.0 {',
 'return unarmed_contact(facts, measure_accuracy_multiplier);'])
must(sources[core + 'combat.rs'], ['fn attack_force(', 'if equip.weapon_is_unarmed() {'])
fatigue = sources[core + 'combat/fatigue.rs']
workload = fatigue.split('pub fn combat_action_workload(', 1)[1].split('pub fn combat_movement_fatigue_per_second(', 1)[0]
must(workload, ['CombatActionWork::Attack => weapon_energy(parameters.attack_travel_metres)',
 '0.5 * weapon_mass_kg.max(0.0) * linear_speed * linear_speed',
 '+ 0.5 * weapon_inertia_kg_m2.max(0.0) * angular_speed * angular_speed',
 'metabolic_joules: mechanical_joules / parameters.mechanical_efficiency'])
must(fatigue, ['(*fatigue + workload.metabolic_joules / parameters.capacity(endurance)).clamp(0.0, 1.0)'])
inv = sources[tcore + 'src/inventory.rs']
must(inv, ['fn striking_item(&self)', 'AttackHand::Main => EquipSlot::HoldingRight',
 'AttackHand::Offhand => EquipSlot::HoldingLeft',
 'return adventuresim_core::equipment::WeaponSkillDistribution::UNARMED;',
 'fn weapon_is_unarmed(&self) -> bool', 'fn weapon_holding_side(&self) -> Option<BodySide>',
 'AttackHand::Main => BodySide::Right', 'AttackHand::Offhand => BodySide::Left',
 'fn weapon_weight(&self) -> f32', 'self.striking_item()', '.unwrap_or_default()',
 'fn weapon_moment_of_inertia(&self) -> f32', '.map_or(0.0, |weapon| weapon.moment_of_inertia_kg_m2)'])
must(sources[tcore + 'src/animation/state.rs'], ['animation_pack: AnimationPackId::HUMANOID_UNARMED',
 'pub fn available_strike_family('])
must(sources[tcore + 'src/player.rs'], ['including the shared unarmed', 'bludgeon fallback'])
must(sources[tcore + 'src/player/combat_state.rs'], ['apply_combat_workload(&mut self.fatigue, workload, endurance, parameters);'])
ingress = sources[tactical + 'ingress.rs']
must(ingress, ['viewer.get_for_attack(event.attacker, event.hand)',
 'melee_attack_capability(&attack_view, &attack_view)', 'skeleton.available_strike_family(event.strike_family)',
 '.begin_attack_timed(', 'charge_started_attack_work('])
must(sources[tactical + 'ingress/work.rs'], ['CombatActionWork::Attack,', 'view.weapon_weight(),',
 'view.weapon_moment_of_inertia(),', 'view.inventory_weight(),', 'view.body_weight(),', 'state.charge_work('])
must(sources[tactical + 'authority.rs'], ['facts.arm_reach <= 0.0', 'facts.weapon_reach < 0.0',
 'if !facts.attack_capability.is_available()'])
must(sources[tactical + 'melee.rs'], ['melee_attack_capability(attacker, attacker)',
 'arm_reach: attacker_dimensions.arm_reach_metres,', 'weapon_reach,'])
must(sources[tactical + 'condition.rs'], ['SkeletonAction::Attack | SkeletonAction::Dodge | SkeletonAction::Block',
 'if !active_action && !moving'])
strategic = sources[module + 'capability.rs']
must(strategic, ['let melee_weapon = hands',
 'item.kind == CatalogItemKind::Weapon && item.melee', 'melee_weapon: self', '.melee_weapon',
 '.map(|item| combat_weapon(item, self.melee_weapon_geometry))'])
must(sources[module + 'capability/combatant.rs'], ['.combat_equipment()',
 'Combatant::from_strategic_state(CombatantStrategicState {'])
must(sources[module + 'character.rs'], ['pub fn unequip_item(',
 'require_strategic_character_authority(ctx, character_id)?;',
 'require_character_no_unresolved_encounter(ctx, character_id)?;',
 'unequip_wearable(ctx, inventory_item_id);'])
must(sources[module + 'strategic/mission_bootstrap.rs'], ['autoresolve_enemy_with_countermeasure(',
 'let outcome = resolve_battle('])
must(sources[module + 'strategic/encounters.rs'], ['autoresolve_enemy(', 'let outcome = resolve_battle('])

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw)
    assert entry['git_blob'] == item['git_blob']
    old = json.loads(raw)
    assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0
    assert old['current_source_reconciled'] is False
    receipts[item['original_path']] = item
assert len(receipts) == 2
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
assert 'no inventednomeleeweaponmeansfistsdisabledclaim' in ledger['segovb_continuation']['lens12']['pending_connected_contracts'][827]
delegated = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
for index, number in ((245, 143), (249, 146)):
    item = receipts[f'segovb/READ-L13-PARALLEL-{number}.json']
    assert delegated[index]['receipt'] == item['original_path']
    assert delegated[index]['receipt_sha256'] == item['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
for i in range(891, 901):
    assert roles[i]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-891-900.json'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 2
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name
               for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 27 Git-bound sources, prior caution and two receipts; no product ran')
