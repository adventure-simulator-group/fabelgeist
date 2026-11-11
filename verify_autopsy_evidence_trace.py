"""Verify bounded autopsy source, prior receipts and links without product execution."""
import gzip
import hashlib
import json
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-AUTOPSY-EVIDENCE-95E.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1 and doc['source_revision']==revision
current={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    head,path=line.split('\t',1);_,kind,oid=head.split()
    assert kind in ('blob','commit') and path not in current
    current[path]=oid

def preserved(item):
    path=(root/item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==item['sha256'] and len(raw)==item['bytes']
    return raw

def must(body,markers):
    for marker in markers:
        if marker not in body: raise ValueError('missing source marker: '+marker)

sources={}
for item in doc['current_sources']:
    raw=preserved(item);oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert item['revision']==revision and oid==item['git_blob']==current[item['path']]
    assert len(raw.splitlines())==item['lines']
    assert all(1<=a<=b<=item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']]=raw.decode()
assert len(sources)==12
core='crates/adventuresim-core/src/'
module='crates/adventuresim-stdb-module/src/'
a=sources[core+'autopsy.rs']
must(a,['pub const MAX_BODY_INJURIES: usize = 32;',
 'entry.defender_id == combatant.id', 'entry.cut_damage > 0.0 || entry.blunt_damage > 0.0',
 'injuries.truncate(MAX_BODY_INJURIES);', 'let injury = strongest_injury(injuries)?;',
 'injury.cut_damage > injury.blunt_damage * 1.5',
 'injury.blunt_damage > injury.cut_damage * 1.5',
 '"mixed tearing and crushing trauma"', '"threats capable of both tearing and forceful impact"',
 'let scene = if context.at_scene && !internal',
 'At the undisturbed scene, blood distribution supports that the wound occurred here.'])
must(sources[core+'physiology.rs'],['pub enum BodyRegion {','pub const ALL: [Self; REGION_COUNT]'])
surgery=sources[module+'surgery.rs']
must(surgery,['fn blank_injury(character_id: u64, limb: BodyRegion)',
 'cut_damage: 0.0,', 'bruise_damage: 0.0,', 'for limb in BodyRegion::ALL',
 '.insert(blank_injury(character_id, limb));'])
character=sources[module+'character.rs']
must(character,['pub fn transition_character_to_dead_at(',
 'crate::corpse::persist_character_death_corpse(',
 'pub(crate) fn insert_persistent_field_character(',
 'crate::surgery::initialize_character_injuries(ctx, character.id);'])
corpse=sources[module+'corpse.rs']
must(corpse,['pub struct CorpseInjury {','pub fn examine_corpse(',
 'let context = autopsy_evidence_context(corpse, minute);',
 'surgery_finding(&injuries, check, context, internal)',
 'bestiary_finding(&injuries, check, context, internal)',
 '.map(|row| row.finding)',
 'let finding = realized_finding(', 'finding: finding.wording,'])
enemy=corpse.split('pub(crate) fn persist_autoresolve_enemy_corpses(',1)[1].split('pub(crate) fn persist_body(',1)[0]
must(enemy,['is_lethal_body(enemy)', 'transition_character_to_dead(',
 'let body = post_combat_body(enemy, &outcome.log);',
 'state.health = body.health.to_vec();', 'state.blood_loss_fraction = body.blood_loss_fraction;'])
assert 'body.injuries' not in enemy and '.corpse_injury()' not in enemy
death=corpse.split('pub(crate) fn persist_character_death_corpse(',1)[1].split('fn body_part_label(',1)[0]
must(death,['.limb_injury()', '.filter(character_id)', 'cut_damage: injury.cut_damage,',
 'blunt_damage: injury.bruise_damage,', '.corpse_injury().insert(CorpseInjury {'])
assert '.filter(|injury|' not in death
context=sources[module+'corpse/autopsy_context.rs']
must(context,['corpse.discovered_minute,','corpse.buried,','corpse.exhumed,',
 'at_scene: location == CorpseLocation::Scene,'])
enc=sources[module+'strategic/encounters.rs']
commit=enc.split('fn commit_autoresolve_outcome(',1)[1].split('fn resolve_random_encounter_battle(',1)[0]
must(commit,['for member in &outcome.allies {','exchange.defender_id == member.id', 'crate::surgery::commit_hit_injury('])
assert 'outcome.enemies' not in commit
must(enc,['persist_autoresolve_enemy_corpses(', '&encounter.archetype,'])
mission=sources[module+'strategic/mission_bootstrap.rs']
must(mission,['pub fn autoresolve_mission(', 'if outcome.resolution != BattleResolution::AlliesVictory',
 'persist_autoresolve_enemy_corpses(', 'Some(&case_site.id)'])
must(sources[module+'world_actor.rs'],['pub(crate) fn materialize_context_roster(',
 'crate::character::insert_persistent_field_character('])
must(sources[module+'outbreak.rs'],['Some(OutbreakPatientDeathKind::Disease) => crate::character::DeathCause::Disease',
 'crate::character::transition_character_to_dead_at(', 'corpse.discovered_minute = discovered_at;'])
must(sources['crates/strategic-web/src/routes/quests.rs'],['SELECT * FROM backend_corpses WHERE owner_character_id', 'corpse.location == "scene"'])
must(sources['crates/strategic-web/src/templates/settlement/character_health.rs'],['@for finding in &corpse.findings { li { (finding) } }'])

# Symbolic branch logic only, never call a product function.
cut=blunt=0.0
assert not (cut>blunt*1.5) and not (blunt>cut*1.5)
assert 3.0*2000.0>=2500

manifest=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))
prior={}
for receipt in doc['recovered_historical_receipts']:
    entry=next(x for x in manifest['entries'] if x['path']==receipt['original_path'])
    raw=preserved(receipt)
    assert entry['sha256']==receipt['sha256'] and entry['bytes']==len(raw)
    prior[receipt['original_path']]=json.loads(raw)
assert len(prior)==6
core_prior=prior['parallel-l12-20261029/READ-CONNECTED-PAR-L12-804-CORE-CORPSE-OWNER.json']
assert core_prior['rows'][0]['assessed_ranges']==[[1,105]] and core_prior['accepted_unique_count_change']==0
native_prior=prior['parallel-l12-20261029/READ-CONNECTED-PAR-L12-804-NATIVE-CORPSE-CONTRACTS.json']
assert native_prior['accepted_unique_count_change']==0 and native_prior['current_source_reconciled'] is False
lens13=prior['parallel-l13-20261029/READ-PL13-242.json']
assert lens13['accepted_unique_count_change']==0 and lens13['current_reconciled'] is False
assert any(x['proposed_id']=='PL13-242-CORPSE-BODY-PATHOLOGY-ADMISSION-PENDING' for x in lens13['connected_contracts'])
queue=json.loads((root/'baseline-059/queue.json').read_text())['entries']
for ref in doc['historical_queue_links']:
    i=int(ref['pointer'].split('/')[-1]); assert queue[i]['contract_id']==ref['id']
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
for ref in doc['historical_sources']:
    role=work['roles'][ref['original_index']]
    assert role['source_id']==ref['source_id'] and role['historical_sha256']==ref['historical_sha256']
    assert ref['byte_identical']==(ref['historical_sha256']==ref['current_sha256'])
assert work['roles'][856]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-856.json'
assert work['roles'][1090]['status']=='full-historical-body-trace-open'
assert work['roles'][1090]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1090-1092.json'
base=json.loads((root/'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
for candidate in doc['candidates']:
    assert sum(x['id']==candidate['id'] and x['record']==name for x in base['newly_assessed_ledger']['entries'])==1
assert progress['new_current_ledger_candidates']==len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled']==0
assert 'ASSESSMENT-L01-HISTORICAL-856.json' in progress['post_transition_historical_body_reads']
recon=json.loads((root/'RECONCILIATION-059.json').read_text())['entries']
for i in (697,2288): assert name in recon[i]['current_reconciliation'] and 'partial' in recon[i]['current_reconciliation']
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified 12 Git-bound sources and six original receipts; no product ran')
