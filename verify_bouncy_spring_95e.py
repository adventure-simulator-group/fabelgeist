#!/usr/bin/env python3
"""Verify read lineage, source identities and symbolic range reasoning only."""
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
a=load('ASSESSMENT-BOUNCY-SPRING-ARITHMETIC-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1310-1318.json')
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
work=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/work['current_tree_inventory']).read_text().splitlines():
 header,path=line.split('\t',1);mode,kind,oid=header.split();tree[path]=oid
s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256']
 assert blob(raw)==x['git_blob']==tree[x['path']] and len(raw.splitlines())==x['lines']
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1310,1319))
assert [x['lines'] for x in h['historical_source_roles']]==[147,200,48,471,29,361,264,134,100]
for x in h['historical_source_roles']:
 role=work['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1310-1318.json'
 assert x['new_personal_read_ranges']==[[1,x['lines']]]
 with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:raw=ar.extractfile(x['sha256']).read()
 assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
 assert sha(cur)==comp['sha256'] and len(cur)==comp['bytes'] and len(cur.splitlines())==comp['lines']
 assert blob(cur)==comp['git_blob']==tree[comp['path']] and (raw==cur)==comp['byte_identical']
assert [x['original_index'] for x in h['historical_source_roles'] if not x['current_comparison']['byte_identical']]==[1311,1315]
config=s['crates/adventuresim-tactical-core/src/combat_config.rs']
block=config.split(b'let bouncy = animation.bouncy_bones;',1)[1].split(b'let ik = animation.inverse_kinematics;',1)[0]
assert b'finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0' in config
assert b'bouncy.frequency_hz' in block and b'bouncy.frequency_hz <= 0.0' in block and b'bouncy.transmission > 1.0' in block
assert b'bouncy.frequency_hz >' not in block
assert b'frequency_hz: 3.0' in s['content/tactical/combat.yaml']
runtime=s['crates/adventuresim-tactical-core/src/combat_config/runtime.rs']
assert b'pub fn install_runtime_snapshot(&self)' in runtime and b'self.validate()?;' in runtime
assert b'snapshot.0.validate()' in s['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'snapshot.0.install_runtime_snapshot()' in s['crates/adventuresim-tactical-netcode/src/client.rs']
spring=s['crates/adventuresim-tactical-client/src/animation/bouncy_bones.rs']
assert b'let omega = std::f32::consts::TAU * frequency.max(0.01);' in spring
assert b'let damped = (omega * omega - decay * decay).max(1e-4).sqrt();' in spring
assert b'spring.offset * damped + b * decay' in spring and b'*spring = Spring { offset, velocity };' in spring
assert b'if spring.at_rest()' in spring and b'if delta_seconds <= 0.0' in spring
assert b'if offset.length_squared() > 1e-8' in spring
assert b'frequency_hz: 3.0' in s['content/tactical/combat.yaml']
primitive=gzip.decompress((r/'native-evidence/rust-nightly-2026-07-13/library/core/src/primitive_docs.rs.gz').read_bytes())
assert b'When a primitive operation (addition, subtraction, multiplication' in primitive
assert b"the result is \xe2\x88\x9e or \xe2\x88\x92\xe2\x88\x9e" in primitive
# Exact integer comparison demonstrates that any multiplier >= 6 applied to
# f32::MAX exceeds the documented overflow threshold; no f32 product is run.
f32_max=(2**24-1)*2**104
assert 6*f32_max > f32_max+2**103
cadence=s['crates/adventuresim-tactical-client/src/animation/authored_cadence.rs']
assert b'.checked_lerp(other.step_distance, amount)?' in cadence and b'speed.clamped_inverse_lerp(' in cadence
catalog=s['crates/adventuresim-tactical-client/src/animation/catalog.rs']
assert b'UnknownMotion(MotionId)' in catalog and b'CatalogError::UnknownMotion(motion.clone())' in catalog
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert x['sha256']==orig['sha256'] and x['bytes']==orig['bytes'] and x['git_blob']==orig['git_blob']
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
expected={357:[1310,1311,1312],358:[1313,1314],359:[1315,1316],360:[1317,1318,1319,1320]}
for batch,ids in expected.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert all(x['original_index'] not in (1319,1320) for x in h['historical_source_roles'])
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1104,1105,1107,1109))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==a['candidates'][0]['id'] and x['lens']==4 for x in ledger)==1
correction=load('CLASSIFICATION-CORRECTION-ARMOR-CHART-95E.json')
assert correction['previous_lens']==6 and correction['corrected_lens']==4
assert next(x for x in ledger if x['id']==correction['candidate_id'])['lens']==4
prior=load('ASSESSMENT-ARMOR-FIXTURE-ADMISSION-95E.json');assert next(x for x in prior['candidates'] if x['id']==correction['candidate_id'])['lens']==4
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical reads, config arithmetic and recorded lens correction; no product execution.')
