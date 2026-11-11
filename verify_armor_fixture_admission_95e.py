#!/usr/bin/env python3
"""Verify historical file identities and static fixture admission evidence only."""
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
a=load('ASSESSMENT-ARMOR-FIXTURE-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1301-1309.json')
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
work=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/work['current_tree_inventory']).read_text().splitlines():
 header,path=line.split('\t',1);mode,kind,oid=header.split();tree[path]=oid
s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob']==tree[x['path']]
 assert len(raw.splitlines())==x['lines'] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1301,1310))
assert [x['lines'] for x in h['historical_source_roles']]==[131,752,227,219,135,121,216,254,455]
for x in h['historical_source_roles']:
 role=work['roles'][x['original_index']]
 assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1301-1309.json'
 assert x['new_personal_read_ranges']==[[1,x['lines']]]
 with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:raw=ar.extractfile(x['sha256']).read()
 assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
 assert sha(cur)==comp['sha256'] and len(cur)==comp['bytes'] and len(cur.splitlines())==comp['lines']
 assert blob(cur)==comp['git_blob']==tree[comp['path']] and (raw==cur)==comp['byte_identical']
assert [x['original_index'] for x in h['historical_source_roles'] if x['current_comparison']['byte_identical']]==[1305,1307]
fixture=s['crates/adventuresim-tactical-client/examples/armor_fixture/mod.rs']
probe=s['crates/adventuresim-tactical-client/examples/armor_browser_probe.rs']
assert b'node.skin().is_some() && node.mesh().is_some()' in fixture
assert b'let skin = node.skin().unwrap();' in fixture and b'let mesh = node.mesh().unwrap();' in fixture
assert b'let primitive = mesh.primitives().next().unwrap();' in fixture and b'.zip(p.unwrap())' in fixture
assert b'pub async fn prepare_probe(bytes: Vec<u8>)' in probe and b'armor_fixture::load(&bytes)?' in probe
assert b'(y - low) / (high - low)' in fixture and b'let texcoords = surface_chart(&positions);' in fixture
assert b'Never use `unwrap` or `expect` outside tests.' in s['crates/AGENTS.md']
runtime=s['crates/adventuresim-character-creator/src/runtime_equipment.rs']
body_validate=runtime.split(b'fn validate(&self) -> Result<()>',1)[1].split(b'fn fit_body(&self)',1)[0]
assert b'self.texcoords.len() != vertices' in body_validate and b'self.faces.is_empty()' in body_validate
assert b'is_finite' not in body_validate
device=s['crates/fabelgeist-armor/src/gpu/body.rs'];new=device.split(b'pub fn new(gpu: &ArmorGpu, surface: BodySurface)',1)[1].split(b'pub fn',1)[0]
assert b'surface.texcoords.len() != vertices' in new and b'BufferUpload::from_elements(surface.texcoords)' in new and b'is_finite' not in new
assert min((0,0,0))==max((0,0,0))
native=load('native-evidence/gltf-1.4.1/MANIFEST.json');archives={x['name']:x for x in native['crates']}
with tarfile.open(r/'native-evidence/gltf-1.4.1'/archives['gltf']['path'],'r:gz') as ar:
 gltf=ar.extractfile('gltf-1.4.1/src/lib.rs').read();reader=ar.extractfile('gltf-1.4.1/src/mesh/util/mod.rs').read()
assert b'gltf.document.validate()?;' in gltf
assert b'Option<ReadPositionDisplacements' in reader and b'.positions()' in reader and b'(positions, normals, tangents)' in reader
with tarfile.open(r/'native-evidence/gltf-1.4.1'/archives['gltf-json']['path'],'r:gz') as ar:
 mesh=ar.extractfile('gltf-json-1.4.1/src/mesh.rs').read();validate=ar.extractfile('gltf-json-1.4.1/src/validation.rs').read()
assert b'pub positions: Option<Index<accessor::Accessor>>' in mesh and b'pub primitives: Vec<Primitive>' in mesh
assert b'impl<T: Validate> Validate for Option<T>' in validate and b'if let Some(value)' in validate
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert x['sha256']==orig['sha256'] and x['bytes']==orig['bytes'] and x['git_blob']==orig['git_blob']
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob'])
 records[x['original_path']]=json.loads(raw)
expected={352:[1300,1301],353:[1302],354:[1303,1304],355:[1305,1306,1307],356:[1308,1309]}
for batch,ids in expected.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1095,1099))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical reads, current-byte distinctions and fixture source evidence; no product execution.')
