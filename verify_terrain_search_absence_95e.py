#!/usr/bin/env python3
"""Verify audit bytes and symbolic search geometry; not a product test."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction as F
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-TERRAIN-SEARCH-ABSENCE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1384-1388.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1384,1389))
assert [x['lines'] for x in h['historical_source_roles']]==[564,76,256,395,276]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1384-1388.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
flight=current[1386];clearance=current[1387]
assert b'const SEARCH_SAMPLES: usize = 64' in flight and b'const HALF_GRID: i32 = 12' in flight
assert b'if xz.distance(nearest_chord) > 0.04' in flight
assert b'let Some(floor) = clearance_floor_at(checked_scene_point_metres(xz)?)? else {\n                    return Ok(None);' in flight
assert b'.min_by(|left, right|' in flight and b'Some(target) => Ok(target)' in flight
assert b'pub(in crate::animation::procedural) fn terrain_clearance_floor' in clearance and b'.height_at(point)' in clearance
admission=s['crates/adventuresim-tactical-core/src/scene/admission.rs'];assert b'if width < 2' in admission and b'heightmap.iter().any(|h| !h.is_finite())' in admission
query=s['crates/adventuresim-tactical-core/src/scene/queries.rs'];assert b'if grid.x < 0.0' in query and b'|| grid.x > (self.grid_width() - 1) as f32' in query
config=s['content/tactical/combat.yaml'];assert b'run_first_release_owner_step_metres: 0.094' in config and b'measured_ankle_sole_offset_metres: 0.085' in config and b'run_swing_minimum_sole_clearance_metres: 0.051' in config
m=load('native-evidence/bevy_math-0.19.1/MANIFEST.json');assert [582,593] in m['personally_inspected_ranges']['bevy_math-0.19.1/src/direction.rs']
with tarfile.open(r/'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate','r:gz') as ar:
 direction=ar.extractfile('bevy_math-0.19.1/src/direction.rs').read();assert b'length.is_finite() && length > 0.0' in direction
# Algebra over a documented hypothetical grid and search indices, no engine or search execution.
step=F(47,500);spacing=step/12;corridor=F(1,25);edge=-F(1,10);start_x=-F(19,200)
assert start_x-5*spacing<edge and (5*spacing)**2+spacing**2<corridor**2
assert (5*spacing)**2+(2*spacing)**2>corridor**2 and 6*spacing>corridor
assert start_x+4*spacing>edge+F(3,100) and (4*spacing)**2<corridor**2
assert (4*spacing)**2+(6*spacing)**2+F(1,50)**2<step**2
assert -F(16,1000)+F(85,1000)+F(51,1000)==F(12,100)>step
assert -F(116,1000)+F(85,1000)+F(51,1000)==F(1,50)
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {391:[1382,1383,1384],392:[1385,1386,1387],393:[1388]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
question=a['conditional_findings'][0];assert question['id']=='REVIEW-TERRAIN-DISK-SEARCH-ABSENCE-001' and question['lens']==5
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1179,1184))
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==question['id'] for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical reads and qualified symbolic terrain-search evidence; no product execution.')
