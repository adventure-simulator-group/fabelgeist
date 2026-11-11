"""Verify source-only standalone mission and attributed historical evidence."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L12-STANDALONE-MISSION-95E.json';doc=load(name)
identity='SEGOVB-L12-STANDALONE-MISSION-ID-PRIVATE-READMISSION'
assert doc['source_revision']==revision and doc['lens']==12 and [c['id'] for c in doc['candidates']]==[identity]
tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta,path=line.split('\t',1);tree[path]=meta.split()[2]
contents={}
for row in doc['current_sources']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']==tree[row['path']]
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines']
    assert row['revision']==revision and all(1<=a<=b<=row['lines'] for a,b in row['ranges'])
    contents[row['path']]=raw
bootstrap=contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
seed=bootstrap.split(b'pub fn seed_standalone_tactical_mission(',1)[1].split(b'\n}',1)[0]
assert b'simulation_bootstrap_authorized(' in seed and b'parse_tactical_enemy_fixture(' in seed
assert seed.index(b'.is_some()') < seed.index(b'MissionId::new(mission_id.clone())')
assert seed.index(b'MissionId::new(mission_id.clone())') < seed.index(b'StandaloneMissionFamily::from_mission_id(&mission_id)')
assert b'case_site_distance_m()' in seed and b'configure_diagnostic_player' in seed
family=contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap/standalone.rs']
assert b'fn from_mission_id(mission_id: &str)' in family
assert b'MissionId::new(mission_id)' in family and b'.split_once(\':\')' in family
assert b'animation_coordinate.is_empty()' in family and b'diagnostic_coordinate.is_empty()' in family
assert b'StandaloneMissionFamily::from_mission_id("mission:animation-demo")' in family
core=contents['crates/adventuresim-core/src/mission.rs']
for s in (b'value.len() > 128',b'!value.starts_with($prefix)',b'value.len() == $prefix.len()',b'authority_id!(MissionId, "mission:")'):
    assert s in core
roster=contents['crates/adventuresim-stdb-module/src/strategic/mission_roster.rs']
for s in (b'context_character_ids(ctx, &self.id)',b'roster.enemy_count().get() != self.enemy_count',
          b'group.capture_enemy_roster(ctx)?.into_enemy_ids()',b'MissionEnemyRoster::try_from(self.enemy_character_ids.clone())',
          b'ctx.db.character().id().find(*id).is_none()'):
    assert s in roster
core_roster=contents['crates/adventuresim-core/src/mission/enemy_roster.rs']
for s in (b'NonZeroU32::new(count)',b'BTreeSet<_>>().len() != enemy_ids.len()',b'u32::try_from(length)'):
    assert s in core_roster
assert b'.enemy_roster(ctx)' in bootstrap and b'.enemy_roster(ctx)' in contents['crates/adventuresim-stdb-module/src/tactical.rs']
transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records=doc['recovered_historical_records'];assert len(records)==11;by={}
for row in records:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());m=transport[row['original_path']]
    assert (row['sha256'],row['git_blob'],row['bytes'])==(m['sha256'],m['git_blob'],m['bytes'])
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if row['redacted_local_path_occurrences']:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else:assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    by[row['original_path']]=(row,json.loads(raw))
def original_sha(path):return by[path][0]['sha256']
r=by['segovb/READ-L12-302.json'][1]
assert r['lens']==12 and [x['original_index'] for x in r['rows']]==list(range(1228,1235))
assert r['accepted_unique_count_change']==0 and r['current_source_reconciled'] is False and r['implementation_gate_complete'] is False
assert original_sha(r['plan'])==r['plan_sha256']
for row in r['rows']:
    assert original_sha(row['registration'])==row['registration_sha256']
    assert row['fresh_personal_body_reads']==row['assessed_ranges'] and row['other_lens_verdict_transfer'] is False
v=by['segovb/VERIFY-L12-302.json'][1]
assert v['receipt_sha256']==original_sha('segovb/READ-L12-302.json')
assert v['state_sha256']==original_sha('segovb/STATE-L12-302.json')
# The previously published 301 chain remains a distinct earlier Lens12 source.
prior=load('ASSESSMENT-L12-THREAT-MATERIALIZATION-95E.json')['recovered_historical_records']
assert len([r for r in prior if 'PARTIAL-L12-301-' in r['original_path']])==5
assert len([r for r in prior if r['original_path']=='segovb/READ-L12-301.json'])==1
histname='ASSESSMENT-L01-HISTORICAL-1227-1234.json';history=load(histname)['historical_source_roles']
assert [r['original_index'] for r in history]==list(range(1227,1235));roles=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read()
        assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']];assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
        assert role.get('current_observations',[])==row['prior_current_observations']
q=load('RECONCILIATION-059.json')['entries']
for i in (242,992,993,995,996):assert q[i]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];matches=[r for r in ledger if r['id']==identity]
assert len(matches)==1 and matches[0]['record']==name
progress=load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_string_traces'] and name in progress['new_nonfinding_assessments']
assert progress['new_current_ledger_candidates']==len(ledger) and progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified standalone mission and roster source evidence; no product ran')
