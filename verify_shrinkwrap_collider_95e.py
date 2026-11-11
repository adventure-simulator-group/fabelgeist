#!/usr/bin/env python3
"""Verify bounded source identities and prior attribution, not product behavior."""
import gzip,hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());contents=lambda p:gzip.decompress((root/p).read_bytes());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-SHRINKWRAP-COLLIDER-95E.json');tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
sources={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);reg=(root/x['registration']).read_bytes();entry=next(y for y in json.loads(reg)['support'] if y['path']==x['path']);assert sha(reg)==x['registration_sha256'] and sha(b)==x['sha256']==entry['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid(b)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);sources[x['path']]=b
h=load('ASSESSMENT-L01-HISTORICAL-2302-2305.json');item=next(x for x in h['historical_source_roles'] if x['original_index']==2304);shrink=contents(item['current_comparison']['preserved_bytes']);assert oid(shrink)==tree[item['current_comparison']['path']]
collider=sources['crates/fabelgeist-hair/src/collider.rs'];strands=sources['crates/fabelgeist-hair/src/strands.rs'];vec=sources['crates/fabelgeist-math/src/vector/vec3.rs']
for x in (b'let locked = j == 0 && self.lock_roots',b'let (distance, out) = collider.query(p)',b'if locked || distance >= offset',b'let target = p + out * (offset - distance)',b'moves.push((target - p) * weight)',b'any |= weight > 0.0',b'if !any',b'for (p, m) in strand.iter_mut().zip(moves.iter())'):assert x in shrink
assert b'(f32::INFINITY, Vec3::up())' in collider and b'if n < 3' in strands and b'Self::new(self.x * rhs, self.y * rhs, self.z * rhs)' in vec
h=load('ASSESSMENT-HAIR-NATIVE-OPERATIONS-95E.json');r=next(x for x in h['recovered_historical_records'] if x['original_path']=='adler/READ-L11-369.json');prior=json.loads(contents(r['preserved_bytes']));role=next(x for x in prior['role_dispositions'] if x['source_indices']==[2304]);assert '0*Inf' in role['disposition']
q=load('RECONCILIATION-059.json')['entries'][1845];assert q['current_reconciliation'].startswith('partial ') and 'ASSESSMENT-SHRINKWRAP-COLLIDER-95E.json' in q['current_reconciliation']
l=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];row=next(x for x in l if x['id']=='REVIEW-HAIR-SHRINKWRAP-MIXED-INFINITE-DISTANCE-95E');assert row['record']=='ASSESSMENT-SHRINKWRAP-COLLIDER-95E.json' and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(l) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified shrinkwrap source and earlier mixed-distance qualification; no product execution.')
