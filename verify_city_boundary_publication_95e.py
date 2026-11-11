#!/usr/bin/env python3
"""Verify city/boundary source evidence and original attribution; no product execution."""
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
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-CITY-BOUNDARY-PUBLICATION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1502-1510.json');w=load('COVERAGE-L01-ROLES.json');proof=load('CITY-BOUNDARY-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1502,1511)) and [x['lines'] for x in h['historical_source_roles']]==[596,43,163,101,153,41,355,153,75]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1502-1510.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
buildings=current[1502];boundary=current[1503];projection=current[1506];assembly=current[1508];gpuinput=current[1509];pack=current[1510];prepared=s['crates/adventuresim-tactical-client/src/presentation/buildings/prepared.rs'];stream=s['crates/adventuresim-tactical-client/src/presentation/buildings/streaming.rs'];status=s['crates/adventuresim-tactical-client/src/strategic_scene/status.rs'];client=s['crates/strategic-web/static/strategic-scene.js'];regional=s['crates/adventuresim-tactical-client/src/presentation/regional_map/city.rs'];focused=s['crates/adventuresim-tactical-client/src/presentation/buildings/focused.rs'];compound=s['crates/adventuresim-tactical-core/src/scene_input/compounds.rs'];mod=s['crates/adventuresim-tactical-client/src/presentation/buildings/gpu/mod.rs'];geo=s['crates/adventuresim-tactical-client/src/presentation/buildings/gpu/geometry.rs']
assert b'interior: (detail == BuildingDetail::Dynamic).then' in buildings and b'interior: Some(' in prepared and b'batches.insert_boundary(boundary, BoundaryPlacement::GateRelative, &materials)?;' in boundary and boundary.index(b'batches.insert_boundary')<boundary.index(b'commands\n')
assert projection.index(b'let Some(projection) = self.0.take()')<projection.index(b'GeneratedBoundary::project(compound, terrain)') and b'projection.scene_digest != environment.scene_digest' in projection and b'terrain.property_foundation(compound.id).ok_or_else' in compound
assert projection.index(b'batches.insert_boundary(')<projection.index(b'commands\n            .spawn(')<projection.index(b'for batch in batches {') and b'let pose = datum.door(door)?;' in projection
assert b'let mut pending = std::mem::take(' in assembly and b'if let Some(publication) = &pending.publication' in assembly and b'.insert(CityAssemblyFailure(error))' in assembly and b'AssemblyError::MissingMesh' in pack
assert b'.get_mut(crate::presentation::ownership::PresentationOwner::Scene)' in stream and b'.set_publication(' not in stream and b'pub(super) publication: Option<CityPublication>' in gpuinput
assert focused.index(b'queued.set_publication(')<focused.index(b'for placement in &city.input().buildings')<focused.index(b'*cities.owners.get_mut(owner) = queued')
assert b'world.get::<CityAssemblyFailure>(pending.root)' in regional and b'InstallationFailure::Assembly' in regional
assert b'city.total == 0 || crate::presentation::city_gpu_ready(PresentationOwner::Scene)' in status and b'city.and_then(|city| city.failure().map(str::to_owned))' in status and b'if (state.error)' in client and client.index(b'if (state.error)')<client.index(b'const ready =')
assert b'READY.get(owner).store(false' in mod and b'.expect("generated city vertex attribute")' in geo and b'city geometry must contain whole triangles' in geo
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(456,[1502]),(457,[1503,1504,1505]),(458,[1506,1507]),(459,[1508,1509,1510])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-214.json']);check=json.loads(records['adler/VERIFY-L11-214.json']);assert [x['original_index'] for x in receipt['sources']]==list(range(1502,1520)) and receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256']==m['adler/READ-L11-214.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-214.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1290,1291,1292,1293));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1290,1291,1292,1293)]==[1502,1504,1506,1508]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==2 and all(sum(x['id']==c['id'] and x['record']=='ASSESSMENT-CITY-BOUNDARY-PUBLICATION-95E.json' for x in ledger)==1 for c in a['conditional_findings']) and all(c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical city/boundary bodies, earlier attribution and bounded publication source connections; no product execution.')
