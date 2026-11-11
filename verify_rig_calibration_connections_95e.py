#!/usr/bin/env python3
"""Verify frozen sources and bounded connections, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-RIG-CALIBRATION-CONNECTIONS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1340-1349.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1340,1350))
assert [x['lines'] for x in h['historical_source_roles']]==[434,80,55,98,103,192,63,69,297,98]
historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1340-1349.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];historical[x['original_index']]=raw
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
assert b'ordered.sort_by(|left, right|' in historical[1345] and b'.entry(family.clone())' in historical[1345]
assert b'joint_name.eq_ignore_ascii_case(&name)' in s['crates/adventuresim-tactical-client/src/animation/pose_buffer.rs']
assert b'let role = BoneRole::from_name(&name)' in s['crates/adventuresim-tactical-client/src/animation/rig_identity.rs']
proc=s['crates/adventuresim-tactical-client/src/animation/procedural/rig.rs'];assert b'Self::ThighLeft => RigJointName::L_UPLEG' in proc and b'.find(|role| &role.joint_name() == name)' in proc
names=s['crates/fabelgeist-rig/src/names.rs'];assert b'Self(Cow::Borrowed("l_upleg"))' in names
lib=s['crates/fabelgeist-rig/src/lib.rs'];assert b'pub struct RigJointName(Cow' in lib and b'eq_ignore_ascii_case' in lib
assert b'definition.joint_ordinal(&name)' in gzip.decompress((r/'main-advance-95e/inertial-evidence/current-1334.source.gz').read_bytes())
assert b'values.copy_from_slice(&replacement)' in historical[1342]
assert b'baked.offset_translations(offsets)' in historical[1348] and b'measure_authored_contact_step_distance(' in historical[1348]
revision=s['crates/adventuresim-tactical-client/src/animation/clip_revision.rs']
for x in [b'caches.bank.invalidate_assets(&changed)',b'caches.strides.invalidate_assets(&changed)',b'character.measurements.invalidate_assets(&changed)',b'rig.invalidate_clip_revisions(&changed)']:
 assert x in revision
cad=s['crates/adventuresim-tactical-client/src/animation/authored_cadence.rs'];assert b'self.phase_curves.remove(&motion)' in cad and b'changed.contains(id)' in cad
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
for batch,ids in {371:[1338,1339,1340],372:[1341,1342,1343,1344],373:[1345,1346,1347],374:[1348,1349]}.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert [x['id'] for x in a['existing_finding_connections']]==['ADLER-L11-POSE-LEG-ROLE-REDECODING','ADLER-L11-LOCOMOTION-ANCHOR-REPARAMETERIZATION']
assert a['new_unique_candidate_count']==0
q=load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [36,37,*range(1135,1145)])
assert 'ASSESSMENT-POSE-LEG-BOUNDS-95E.json' in q[36]['current_reconciliation']
assert 'ASSESSMENT-INERTIAL-COEFFICIENTS-95E.json' in q[37]['current_reconciliation']
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified ten historical reads and two earlier rig/calibration connections; no product execution.')
