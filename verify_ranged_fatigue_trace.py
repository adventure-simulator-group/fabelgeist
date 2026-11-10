"""Verify source-bound ranged-work evidence and earlier receipt, not product behavior."""
import gzip
import hashlib
import json
from pathlib import Path
root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-RANGED-FATIGUE-95E.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1 and doc['source_revision']==revision
tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    head,path=line.split('\t',1);_,kind,oid=head.split()
    assert kind in ('blob','commit') and path not in tree
    tree[path]=oid

def preserved(item):
    path=(root/item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==item['sha256'] and len(raw)==item['bytes']
    return raw

def must(body,markers):
    for marker in markers:
        if marker not in body:raise ValueError('missing source marker: '+marker)

sources={}
for item in doc['current_sources']:
    raw=preserved(item);oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert item['revision']==revision and oid==item['git_blob']==tree[item['path']]
    assert len(raw.splitlines())==item['lines']
    assert all(1<=a<=b<=item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']]=raw.decode()
assert len(sources)==17
must(sources['AGENTS.md'],['Weapon work, defenses, dodges, and strenuous movement all contribute to it.',
 'Combat performance depends on total incapacitation, not fatigue directly.',
 'Neither fatigue nor total incapacitation adds a movement-speed penalty'])
must(sources['content/tactical/combat.yaml'],['recovery_watts_per_endurance: 55.0','capacity_joules_per_endurance: 16000.0',
 'combat_round_seconds: 1.0'])
core='crates/adventuresim-core/src/'
module='crates/adventuresim-stdb-module/src/'
tactical='crates/adventuresim-tactical-server/src/'
a=sources[core+'autoresolve.rs']
must(a,['fatigue: 0.0,','active_work_seconds: AnimationElapsed::ZERO,',
 '+ self.fatigue','fn charge_action_work(&mut self, work: CombatActionWork, duration_seconds: AnimationElapsed)',
 'self.active_work_seconds = self.active_work_seconds.max(duration_seconds);',
 'elapsed_seconds.saturating_sub(self.active_work_seconds)',
 'resolve_ranged_round(allies, enemies, round, random, recorder, parameters)?;',
 '.for_each(|combatant| combatant.advance_condition(parameters.combat_round_seconds));',
 'MeleeResponseChoice::None,'])
opening=a.split('fn take_opening_volley_step(',1)[1].split('fn resolve_ranged_round(',1)[0]
plan=a.split('fn plan_ranged_round(',1)[1].split('fn apply_attack_result(',1)[0]
apply=a.split('fn apply_pending_attacks(',1)[1].split('fn resolve_opening_volleys(',1)[0]
effect=a.split('fn apply_attack_result(',1)[1].split('fn preferred_attack_mode(',1)[0]
for body in (opening,plan,apply,effect):
    assert 'charge_action_work(' not in body and 'charge_defensive_work(' not in body
must(opening,['equipment.ammunition -= 1;', 'autoresolve_optimal_ranged_exchange(', 'MeleeResponseChoice::None,'])
must(plan,['autoresolve_optimal_ranged_exchange(', 'attacker.equipment.ammunition -= 1;'])
must(apply,['MeleeResponseChoice::None,'])
optimal=a.split('fn autoresolve_optimal_ranged_exchange(',1)[1].split('fn autoresolve_hit_precision(',1)[0]
must(optimal,['attacker: &Combatant,','defender: &Combatant,','DefenderResponse::None,',
 'DefenderResponse::Dodge {','DefenderResponse::Block { effectiveness: 1.0 }',
 '[dodge, block].into_iter().fold(neutral'])
assert 'charge_action_work(' not in optimal and 'charge_defensive_work(' not in optimal
must(sources[core+'combat/fatigue.rs'],['CombatActionWork::Attack => weapon_energy',
 'CombatActionWork::WeaponDefense => weapon_energy','CombatActionWork::ExplosiveDodge =>',
 'pub fn apply_combat_workload(', 'pub fn recover_combat_fatigue('])
must(sources[core+'autoresolve/melee_round/mod.rs'],['charge_action_work(CombatActionWork::Attack, attack_duration)',
 'charge_defensive_work(defender, response)?;'])
must(sources[core+'autoresolve/melee_round/commitment.rs'],['defender.charge_action_work(CombatActionWork::WeaponDefense, duration)',
 'defender.charge_action_work(CombatActionWork::ExplosiveDodge, duration)'])
must(sources[module+'strategic/battle_combatants.rs'],['crate::capability::load_combatant('])
must(sources[module+'capability/combatant.rs'],['let fatigue = fatigue_incapacitation', 'Combatant::from_strategic_state(CombatantStrategicState {'])
must(sources[module+'capability.rs'],['.find(|item| item.kind == CatalogItemKind::Weapon && item.ranged)',
 'ranged_weapon: self', 'ammunition: self.ammunition,'])
must(sources[module+'strategic/mission_bootstrap.rs'],['let outcome = resolve_battle(allies, enemies, seed, opening)?;'])
ingress=sources[tactical+'combat/ingress.rs']
must(ingress,['charge_started_attack_work(', 'RangedActionRequest::Start { target }',
 'cmd.trigger(RangedAttackStartedIntent {'])
start=ingress.split('pub(super) fn on_ranged_attack_started(',1)[1].split('pub(super) fn authoritative_line_of_sight(',1)[0]
must(start,['.begin_attack_timed(spec, start, timing.contact, timing.end)', 'authority.observe('])
assert 'charge_started_attack_work(' not in start and 'TacticalCombatState' not in start and '.charge_work(' not in start
must(sources[tactical+'combat/ingress/work.rs'],['CombatActionWork::Attack,','state.charge_work('])
condition=sources[tactical+'combat/condition.rs']
must(condition,['SkeletonAction::Attack | SkeletonAction::Dodge | SkeletonAction::Block',
 'combat_movement_fatigue_per_second(', 'if !active_action && !moving', 'recover_combat_fatigue('])
ranged=sources[tactical+'combat/ranged.rs']
must(ranged,['combat_incapacitation_performance(attacker.incapacitation)?',
 'ammunition.consume(&mut cmd, &q_ids, &mut consequences, &shot);',
 'attacker_view.resolve_ranged_attack('])
assert '.charge_work(' not in ranged and 'apply_combat_workload(' not in ranged
must(sources[tactical+'bot/offense/ranged.rs'],['cmd.trigger(RangedAttackStartedIntent {'])
must(sources['crates/adventuresim-tactical-core/src/player/combat_state.rs'],['pub fatigue: f32,',
 'apply_combat_workload(&mut self.fatigue, workload, endurance, parameters);',
 'fatigue: self.fatigue.clamp(0.0, 1.0),'])
must(sources['crates/adventuresim-tactical-core/src/animation/state.rs'],['Self(ActionKind::Attack { .. }) => SkeletonAction::Attack',
 'pub fn begin_attack_timed(', 'self.action = ActionState(ActionKind::Attack {'])
manifest=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))
receipt=doc['recovered_historical_receipts'][0]
entry=next(x for x in manifest['entries'] if x['path']==receipt['original_path'])
raw=preserved(receipt);assert entry['sha256']==receipt['sha256'] and entry['bytes']==len(raw)
old=json.loads(raw);assert old['lens']==1 and old['accepted_unique_count_change']==0
assert any('fatigue carryover' in x for x in old['pending'])
queue=json.loads((root/'baseline-059/queue.json').read_text())['entries']
for item in doc['historical_queue_links']:
    i=int(item['pointer'].split('/')[-1]);assert queue[i]['contract_id']==item['id']
role=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles'][857]
ref=doc['historical_source_roles'][0]
assert role['source_id']==ref['source_id'] and role['historical_sha256']==ref['historical_sha256']
assert role['historical_assessment']==ref['historical_assessment'] and role['current_observations']
base=json.loads((root/'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
for c in doc['candidates']:
    assert sum(x['id']==c['id'] and x['record']==name for x in base['newly_assessed_ledger']['entries'])==1
assert progress['new_current_ledger_candidates']==len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled']==0
rec=json.loads((root/'RECONCILIATION-059.json').read_text())['entries']
for i in (702,1563):assert name in rec[i]['current_reconciliation'] and 'partial' in rec[i]['current_reconciliation']
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified 17 Git-bound sources and one historical receipt; no combat or product ran')
