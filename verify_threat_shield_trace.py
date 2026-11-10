"""Verify the authored-shield source trace, without running the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-THREAT-SHIELD-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid


def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)


sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == tree[item['path']]
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 18
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/strategic/'
bestiary = sources['content/quests/bestiary.yaml']
watch = bestiary.split('"id": "town_watch"', 1)[1].split('"id": "armed_retainer"', 1)[0]
retainer = bestiary.split('"id": "armed_retainer"', 1)[1].split('"id":', 1)[0]
assert '"protection":"shielded"' in watch and '"protection":"armored"' in retainer
must(sources[core + 'quest_catalog_validation.rs'], ['string(combat, "protection", &at)?',
 '"shielded",', '"armored",'])
must(sources[core + 'bestiary.rs'], ['"shielded" => Protection::Shielded',
 '"armored" => Protection::Armored'])
threat = sources[core + 'autoresolve/threat.rs']
must(threat, ['pub fn authored_threat_combatant(', 'let mut combatant = Combatant::new(id);',
 'equip_threat_weapon(&mut combatant, weapon, profile.ranged)?;',
 'matches!(profile.protection, Protection::Armored)',
 'combatant.equipment.shield_block_bonus = 1.0;',
 'matches!(protection, Protection::Shielded | Protection::Armored)'])
assert threat.count('shield_block_bonus') == 1
assert 'shield_side' not in threat and 'defense_item_id' not in threat
must(sources[core + 'autoresolve.rs'], ['pub fn new(id: u64) -> Self',
 'equipment: CombatEquipment::default(),'])
model = sources[core + 'autoresolve/model.rs']
default = model.split('impl Default for CombatEquipment {', 1)[1].split('impl CombatEquipment {', 1)[0]
must(default, ['shield_block_bonus: 0.0,', 'shield_side: None,', 'defense_item_id: None,'])
must(model, ['fn shield_block_bonus(&self) -> f32', 'self.shield_block_bonus',
 'fn shield_holding_side(&self) -> Option<BodySide>', 'self.shield_side'])
must(sources[core + 'autoresolve/melee_defense.rs'], [
 'defender.equipment.melee_weapon.is_some() || defender.equipment.shield_block_bonus > 0.0',
 'DefenderResponse::Block { effectiveness: 1.0 }', 'defender.equipment.shield_block_bonus,'])
must(sources[core + 'autoresolve/melee_round/commitment.rs'], [
 'defender.equipment.shield_block_bonus > 0.0', 'MeleeDefenseCommitmentKind::TransformedOffhand'])
must(sources[core + 'autoresolve/melee_round/mod.rs'], [
 'let response = shield_aligned_response(', 'defender.equipment.shield_holding_side(),'])
shield = sources[core + 'combat/defense.rs'].split('pub fn shield_aligned_response(', 1)[1].split(
 'pub fn reciprocal_intercept_response(', 1)[0]
must(shield, ['shield_side: Option<BodySide>',
 'let Some(resting_coordinate) = shield_side.and_then(shield_resting_coordinate) else {',
 'return response;', 'let distance = (contact_lateral_coordinate(contact) - resting_coordinate).abs();'])
must(sources[core + 'combat/targeting.rs'], [
 'let shield_bonus = defender_equip.shield_block_bonus();',
 '5.0 * (1.0 - (-(shield_bonus + block_skill) / 2.0).exp())'])
must(sources[core + 'autoresolve/power.rs'], [
 '(1.0 + combatant.equipment.shield_block_bonus.max(0.0))'])
must(sources[core + 'autoresolve/melee_iteration.rs'], [
 'equipment.shield_block_bonus = block;', 'equipment.shield_side = Some(match',
 'equipment.defense_item_id = Some(stable_id(shield_id));'])
must(sources[module + 'autoresolve.rs'], ['adventuresim_core::autoresolve::authored_threat_combatant('])
must(sources[module + 'encounters.rs'], ['autoresolve_enemy(', '&encounter.archetype,',
 'let outcome = resolve_battle('])
must(sources[module + 'mission_bootstrap.rs'], ['autoresolve_enemy_with_countermeasure(',
 '&hostile_group.enemy_type,', 'let outcome = resolve_battle('])
must(sources[module + 'authority_model.rs'], ['autoresolve_enemy(',
 'adventuresim_core::autoresolve::autoresolve_combat_power(&enemy)'])
must(sources[module + 'incidents.rs'], ['"town_watch",', '"armed_retainer",', 'enemy_type,'])

roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id']
    assert role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
for i in range(866, 879):
    assert roles[i]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-866-878.json'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name
               for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 18 Git-bound source records and historical role links; no product ran')
