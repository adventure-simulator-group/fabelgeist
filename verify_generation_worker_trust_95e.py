#!/usr/bin/env python3
"""Verify recovered role, receipt and pinned source evidence; never run product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-GENERATION-WORKER-TRUST-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1541-1548.json');w=load('COVERAGE-L01-ROLES.json');proof=load('WORKER-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed' and not a['conditional_findings']
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
def check_chain(path,p,expected):
 assert [s['name'] for s in p['chain']]==path.split('/');curtree=proof['root_tree']
 for step in p['chain']:
  info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes'];assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
 assert curtree==p['git_blob']==oid('blob',expected) and p['sha256']==sha(expected) and p['bytes']==len(expected)
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1541,1549)) and [x['lines'] for x in h['historical_source_roles']]==[9,69,467,204,164,73,23,67]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1541-1548.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur;check_chain(c['path'],proof['paths'][c['path']],raw)
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
v=current[1545];worker=current[1548];wasm=current[1546];error=current[1547];scene=current[1542]
binary=s['crates/adventuresim-tactical-core/src/geometry_transport/binary.rs'];cache=s['crates/strategic-web/static/strategic-generation-cache.js'];js=s['crates/strategic-web/static/strategic-generation.js'];pool=s['crates/strategic-web/static/strategic-generation-pool.js'];recipe=s['crates/adventuresim-tactical-client/src/presentation/recipe_mesh.rs'];prepared=s['crates/adventuresim-tactical-client/src/presentation/buildings/prepared.rs'];buildings=s['crates/adventuresim-tactical-client/src/presentation/buildings.rs'];life=s['crates/adventuresim-tactical-client/src/presentation/demo_lifecycle.rs'];detail=s['crates/adventuresim-building-generator/src/detail.rs'];closures=s['crates/adventuresim-building-generator/src/lod/closures.rs'];req=s['crates/adventuresim-tactical-client/src/presentation/generation/requests.rs'];inst=s['crates/adventuresim-tactical-client/src/presentation/generation/installation.rs'];owner=s['crates/adventuresim-tactical-client/src/presentation/generation/ownership.rs']
assert b'job.require_owner(ticket)?' in worker and b'if *program == venue.recipe.program' in worker and b'if *program == facade.program' in worker and b'if city.matches(&input, &graphics)' in worker and b'Some(recipe) if recipe.program == program' in v
assert b'bytes.len().is_multiple_of(size_of::<T>())' in binary and b'bytemuck::pod_collect_to_vec(bytes)' in binary and all(x in v for x in (b'positions: Vec<[f32; 3]>',b'normals: Vec<[f32; 3]>',b'uvs: Vec<[f32; 2]>',b'tangents: Vec<[f32; 4]>',b'indices: Vec<u32>',b'mesh.insert_indices(Indices::U32(self.indices))'))
assert recipe.count(b'.vertices\n')>=3 and b'batch.indices.clone()' in recipe and b'mesh.generate_tangents()' in recipe
assert all(x in cache for x in (b'CACHE_FORMAT',b'${revision}',b'MAX_PRODUCT_BYTES',b'digest(record.bytes) !== record.checksum',b'length !== record.decodedSize')) and b'generation/product-decode' in js and b'generation/product-mismatch' in js and b'await cache.remove(job)' in js and b'await Promise.all(' in pool
assert b'v.recipe.program == placement.program' in req and b'.and_then(|v| v.geometry.take())' in inst and b'cache.levels.push(compiled.clone())' in prepared and b'take_venue_geometry(_owner, program)?' in buildings and buildings.index(b'.find(|compiled| compiled.program == *program && compiled.detail == detail)')<buildings.index(b'take_venue_geometry(_owner, program)?')
assert b'cache.recipes.clear()' in life and b'TacticalBuildingMeshCache::default()' in life and b'generation::clear_residency()' in life and b'slot.staged = slot.active.clone()' in owner and b'compile_detail(plan, &dynamic_closure_solids(plan)?)' in detail and b'crate::detail::dynamic_closure_solids(self)?' in closures
assert b'if primary != placement' in scene and b'if *placement != (*distant).into()' in scene and b'PreparedStrategicScene::from_products' in owner and b'exception.set_name(error.code())' in error and b'wasm_receive_job' in wasm
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']);b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(474,[1541,1542,1543]),(475,[1544,1545]),(476,[1546,1547,1548,1549])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False and check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
 for row in receipt['rows']:assert row['registration'] in records and row['registration_sha256']==m[row['registration']]['sha256']
old=load('ASSESSMENT-GENERATION-IDENTITY-95E.json');prior={x['original_path']:x for x in old['recovered_historical_records']};adler=json.loads(contents(prior['adler/READ-L11-218.json']['preserved_bytes']));assert [x['original_index'] for x in adler['sources'] if 1541<=x.get('original_index',0)<=1549]==list(range(1541,1550)) and adler['current_reconciled'] is False and adler['validated'] is False
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1308,1309,1310));p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert 'ASSESSMENT-L01-HISTORICAL-1541-1548.json' in p['post_transition_historical_body_reads'] and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified eight original bodies, attributed earlier receipts and bounded worker/cache source evidence; no product execution.')
