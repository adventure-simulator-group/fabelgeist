"""Verify archived surgery/time source and attributed receipts; no product runs."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009';name='ASSESSMENT-L04-TREATMENT-WAIT-95E.json';doc=load(name)
ids=['REVIEW-TREATMENT-STITCH-POSTWAIT-001','REVIEW-TREATMENT-SPLINT-POSTWAIT-001']
assert doc['source_revision']==rev and doc['lens']==4 and [r['id'] for r in doc['candidates']]==ids
tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta,path=line.split('\t',1);tree[path]=meta.split()[2]
contents={}
for row in doc['current_sources']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']==tree[row['path']]
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines'] and row['revision']==rev
    assert all(1<=a<=b<=row['lines'] for a,b in row['ranges']);contents[row['path']]=raw
surgery=contents['crates/adventuresim-stdb-module/src/surgery.rs']
treat=surgery.split(b'\npub fn treat_limb(',1)[1].split(b'\n}',1)[0]
for value in (b'require_strategic_character_authority(ctx, actor_id)',
              b'SurgeryProcedure::Stitch if injury.cut_damage > 0.0 && !injury.stitched',
              b'SurgeryProcedure::RemoveSplint if injury.splint_inventory_item_id.is_some()',
              b'if !align_and_advance(ctx, actor_id, patient_id, duration)?',
              b'contextual_treatment_decision(',b'injury = injury_for(ctx, patient_id, limb)',
              b'injury.stitched = true',b'injury.stitch_quality = skill',
              b'return_splint(ctx, &mut injury)?',b'.insert(TreatmentActionReceipt'):
    assert value in treat,value
assert treat.index(b'SurgeryProcedure::Stitch if injury.cut_damage > 0.0') < treat.index(b'if !align_and_advance') < treat.rindex(b'injury = injury_for(ctx, patient_id, limb)')
after=treat.split(b'injury = injury_for(ctx, patient_id, limb)',2)[-1]
assert b'SurgeryProcedure::Stitch if injury.cut_damage > 0.0' not in after
assert b'SurgeryProcedure::RemoveSplint if injury.splint_inventory_item_id.is_some()' not in after
settle=surgery.split(b'pub(crate) fn settle_injuries(',1)[1].split(b'\npub fn convalescence_minutes(',1)[0]
for value in (b'recovery.get().min(interval.elapsed)',b'if injury.bandaged',b'natural + 0.01 + stitch_bonus',
              b'injury.splint_inventory_item_id.is_some()',b'natural + FRACTURE_HEALING_PER_DAY',
              b'if injury.fracture_damage == 0.0',b'return_splint(ctx, injury)?'):
    assert value in settle
return_splint=surgery.split(b'fn return_splint(',1)[1].split(b'\n}',1)[0]
assert b'.splint_owner_id' in return_splint and b'.splint_inventory_item_id' in return_splint
assert b'.take()' in return_splint and b'Applied splint has no owner' in return_splint
assert b'item.character_id = owner' in return_splint
assert b'FRACTURE_HEALING_PER_DAY: f32 = 0.0125' in surgery
core=contents['crates/adventuresim-core/src/surgery.rs']
assert b'SurgeryProcedure::Stitch | SurgeryProcedure::OpenBody => 60.0' in core
assert b'SurgeryProcedure::Splint | SurgeryProcedure::RemoveSplint => 45.0' in core
assert b'(base - skill.max(0.0) * 5.0).max(10.0).ceil() as u64' in core
assert b'SELF_TREATMENT_PENALTY: f32 = 2.5' in core
clock=contents['crates/adventuresim-stdb-module/src/time/advancement.rs']
wait=clock.split(b'pub fn advance_character_wait_time_in_plan(',1)[1].split(b'\n}',1)[0]
assert b'settle_injuries(' in wait and b'InjuryRecoveryMinutes::new(elapsed)' in wait
assert b'BASE_HEALTH_RECOVERED_PER_DAY: f32 = 0.01' in contents['crates/adventuresim-stdb-module/src/time/model.rs']
assert b'physiology_check.clamp(0.0, 5.0)' in contents['crates/adventuresim-stdb-module/src/time/settlement_rest.rs']
assert b'pub const MINUTES_PER_DAY: u64 = 24 * 60' in contents['crates/adventuresim-world-schema/src/calendar.rs']
world=contents['crates/adventuresim-stdb-module/src/world_actor.rs']
assert b'if actor_id == patient_id' in world and b'if claim.is_some()' in world
assert b'ContextualActionDecision::Allowed(ContextualActionReason::SelfAction)' in contents['crates/adventuresim-core/src/strategic_action.rs']
disease=contents['crates/adventuresim-stdb-module/src/disease.rs'].split(b'pub fn record_committed_cut(',1)[1].split(b'\n}',1)[0]
assert b'if severity <= 0.0' in disease and b'return Ok(())' in disease
# Arithmetic checks validate the recorded examples, not the game or a fixture.
assert 50/1440*0.02 > 0.0001 and 45/1440*0.0225 > 0.0001
transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']};by={}
assert len(doc['recovered_historical_records'])==14
for row in doc['recovered_historical_records']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());m=transport[row['original_path']]
    assert (row['sha256'],row['git_blob'],row['bytes'])==(m['sha256'],m['git_blob'],m['bytes'])
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if row['redacted_local_path_occurrences']:assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else:assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    by[row['original_path']]=(row,json.loads(raw))
def original_sha(p):return by[p][0]['sha256']
for number,indices in ((306,[1238]),(307,[1239,1240,1241])):
    key=f'segovb/READ-L12-{number}.json';r=by[key][1]
    assert r['lens']==12 and [x['original_index'] for x in r['rows']]==indices
    assert r['accepted_unique_count_change']==0 and r['current_source_reconciled'] is False and r['implementation_gate_complete'] is False
    assert original_sha(r['plan'])==r['plan_sha256']
    for x in r['rows']:
        assert original_sha(x['registration'])==x['registration_sha256'] and x['other_lens_verdict_transfer'] is False
        assert x['fresh_personal_body_reads']==x['assessed_ranges']
    v=by[f'segovb/VERIFY-L12-{number}.json'][1]
    assert v['receipt_sha256']==original_sha(key) and v['state_sha256']==original_sha(f'segovb/STATE-L12-{number}.json')
r=by['segovb/READ-L12-306.json'][1];assert len(r['partial_sequence'])==2
for i,(ref,interval,tail) in enumerate(zip(r['partial_sequence'],[[1,800],[801,1435]],[[[801,1435]],[]])):
    assert original_sha(ref['path'])==ref['sha256'];part=by[ref['path']][1]
    assert part['fresh_personal_body_reads']==[interval] and part['unread_tail']==tail
    assert part['other_lens_coverage_transferred'] is False and part['current_source_reconciled'] is False
histname='ASSESSMENT-L01-HISTORICAL-1238-1241.json';history=load(histname)['historical_source_roles'];roles=load('COVERAGE-L01-ROLES.json')['roles']
assert [r['original_index'] for r in history]==list(range(1238,1242))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read();assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']];assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
q=load('RECONCILIATION-059.json')['entries'];assert q[1006]['contract_id']=='Q-L12-a2ef7026a3b720bc1401' and q[1006]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches=[x for x in ledger if x['id']==identity];assert len(matches)==1 and matches[0]['record']==name
p=load('CONTINUATION-PROGRESS-95E.json')
assert name in p['post_transition_bounded_operation_traces']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified treatment wait source evidence and historical partials; no product ran')
