#!/usr/bin/env python3
"""Verify preserved audit evidence and a symbolic reach example only."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-GUARD-SWING-REACH-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1375-1383.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1375,1384))
assert [x['lines'] for x in h['historical_source_roles']]==[498,148,373,297,564,479,107,58,59]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1375-1383.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
guard=current[1375];reach=current[1376];solver=current[1379];terminal=current[1383];settle=current[1377]
for x in [b'if horizontal_distance_squared > reach_squared',b'(maximum_reach * maximum_reach - vertical * vertical)',b'.clamp_length_max(horizontal_reach)',b'            target.y,',b'if !targets.left.is_finite() || !targets.right.is_finite()']:
 assert x in guard,x
assert b'Self::Extended { upper, lower } => upper.metres() + lower.metres() - 0.0001' in reach
assert b'let distance = target_distance.clamp(minimum_distance, maximum_distance)' in solver
assert b'matches!(self, Self::Prepared(_))' in terminal and b'*self = Self::Seeded { shift }' in terminal
assert settle.index(b'memory.terminal_contacts.seed(memory.pelvis_shift)')<settle.index(b'let left_seed = if memory.terminal_contacts.is_prepared()')
co=load('ASSESSMENT-L01-HISTORICAL-1370-1374.json')['historical_source_roles'][0];coord=gzip.decompress((r/co['current_comparison']['preserved_bytes']).read_bytes())
assert b'validate_guard_frame_targets(' in coord and b'ReachLimit::Extended {' in coord and b'solve_two_bone_with_reach(' in coord
lock=s['Cargo.lock'];assert b'name = "bevy_math"\nversion = "0.19.1"' in lock and b'"glam 0.32.1"' in lock and b'f70749695b063ecbf6b62949ccccde2e733ec3ecbbd71d467dca4e5c6c97cca0' in lock
m=load('native-evidence/glam-0.32.1/MANIFEST.json');assert [917,925] in m['personally_inspected_ranges']['glam-0.32.1/src/f32/vec2.rs']
with tarfile.open(r/'native-evidence/glam-0.32.1/glam-0.32.1.crate','r:gz') as ar:
 vec=ar.extractfile('glam-0.32.1/src/f32/vec2.rs').read();assert b'pub fn clamp_length_max(self, max: f32)' in vec and b'max * (self / math::sqrt(length_sq))' in vec
# Symbolic scalar inequalities and zero budget only; no application or animation execution.
maximum=Fraction(1)-Fraction(1,10000);horizontal_sq=Fraction(4);vertical_sq=Fraction(4)
assert horizontal_sq>maximum*maximum and max(maximum*maximum-vertical_sq,0)==0 and Fraction(2)>maximum
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {387:[1374,1375],388:[1376,1377],389:[1378,1379],390:[1380,1381],391:[1382,1383,1384]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
question=a['conditional_findings'][0];assert question['id']=='REVIEW-GUARD-SWING-VERTICAL-REACH-001' and question['lens']==4
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1170,1179))
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==question['id'] for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical reads and a qualified symbolic reach example; no product execution.')
