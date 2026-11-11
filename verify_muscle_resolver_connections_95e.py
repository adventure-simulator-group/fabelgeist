#!/usr/bin/env python3
"""Verify preserved audit evidence and symbolic bounds without product execution."""
import difflib
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
a=load('ASSESSMENT-MUSCLE-RESOLVER-CONNECTIONS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1398-1400.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1398,1399,1400]
assert [x['lines'] for x in h['historical_source_roles']]==[275,217,143]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1398-1400.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1400:
   diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical/span.rs',tofile='95e/span.rs'))
   assert not comp['byte_identical'] and (r/comp['comparison_diff']).read_text()==diff
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
muscles=current[1398];resolver=current[1399];span=current[1400]
assert b'error.to_scaled_axis()' in muscles and b'.proportional_gain.per_second_squared()' in muscles and b'.clamp_length_max(' in muscles
assert b'if kick.length_squared() <= 1e-8' in muscles and b'parent_spin.0 += parent_delta' in muscles and b'child_spin.0 += child_delta' in muscles
assert b'age_seconds.checked_add(elapsed)' in muscles and b'ragdoll.age_seconds = age' in muscles
assert b'BlendWeight::from(progress.complement())' in span and b'BlendWeight::from(progress)' in span and b'checked_scale(' in span
assert b'let Some(start) = start.and_then' in resolver and b'return Ok(())' in resolver
mod=s['crates/adventuresim-tactical-client/src/animation/mod.rs'];assert b'match sample_resolver.resolve_body(skeleton, &evaluation)' in mod and b'continue;' in mod and b'target.into_playback(&evaluation)' in mod
contrib=s['crates/adventuresim-tactical-client/src/animation/playback_contributions.rs'];assert b'self.clips.is_empty() && self.extrapolated.is_empty() && self.continuations.is_empty()' in contrib
state=s['crates/adventuresim-tactical-client/src/animation/playback_state.rs'];assert b'!self.plan.is_bind()' in state
loading=s['crates/adventuresim-tactical-client/src/animation/loading.rs'];assert b'collect_loaded_grips(&asset_server, &gltfs, &clips, &mut runtime)' in loading and b'runtime.clips.insert(key, clip)' in loading
params=s['crates/adventuresim-tactical-core/src/animation/response_parameters.rs'];assert b'checked_nonnegative(value, AnimationQuantity::AngularStiffness)' in params
content=s['content/tactical/combat.yaml'];assert b'proportional_gain: 600.0' in content and b'maximum_kick_radians_per_second: 60.0' in content
with tarfile.open(r/'native-evidence/glam-0.32.1/glam-0.32.1.crate','r:gz') as ar:
 v=ar.extractfile('glam-0.32.1/src/f32/vec3.rs').read();assert b'max * (self / math::sqrt(length_sq))' in v and b'x: self.x.mul(rhs)' in v
 for p in ('glam-0.32.1/src/f32/scalar/quat.rs','glam-0.32.1/src/f32/sse2/quat.rs'):
  assert b'pub fn to_scaled_axis(self) -> Vec3' in ar.extractfile(p).read()
# Magnitude comparison only: it does not evaluate product, platform, or quaternion behavior.
f32max=Fraction(2**24-1,2**23)*2**127;gain=2*10**38
assert gain<f32max and 2*gain>f32max
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {398:[1397,1398],399:[1399,1400]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
for batch in (38,39):
 receipt=json.loads(records[f'root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-{batch:03}.json'])
 assert all(row['revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' for row in receipt['rows'])
for path in ('adler/READ-L11-198.json','adler/READ-L11-199.json'):assert path in records
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries']
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1193,1196)]==[1398,1399,1400]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [37,1193,1194,1195])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']=='REVIEW-RAGDOLL-MUSCLE-KICK-OVERFLOW-001' for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified three historical reads, source diff and qualified muscle/resolver evidence; no product execution.')
