#!/usr/bin/env python3
"""Verify source and recovery lineage, plus symbolic bounds; no product execution."""
import difflib,gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-UNDERSTORY-LITTER-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1562-1570.json');w=load('COVERAGE-L01-ROLES.json');proof=load('LITTER-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
def check_chain(path,p,expected):
 assert [s['name'] for s in p['chain']]==path.split('/');curtree=proof['root_tree']
 for step in p['chain']:
  info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes'];assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
 assert curtree==p['git_blob']==oid('blob',expected) and p['sha256']==sha(expected) and p['bytes']==len(expected)
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1562,1571)) and [x['lines'] for x in h['historical_source_roles']]==[322,18,964,68,129,122,58,257,78]
current={};historic={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1562-1570.json' and x['new_personal_read_ranges']==([[1,350],[351,675],[676,964]] if i==1564 else [[1,x['lines']]])
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historic[i]=raw;c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert c['source_revision']==rev and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']] and c['byte_identical']==(raw==cur);current[i]=cur;check_chain(c['path'],proof['paths'][c['path']],raw)
  if i in (1562,1564):
   diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode();d=c['exact_diff'];assert diff==contents(d['preserved_bytes']) and sha(diff)==d['sha256'] and len(diff)==d['bytes']
  else:assert raw==cur
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
for x in a['earlier_reinspection_sources']:
 b=contents(x['preserved_bytes']);path=x['source_id'].split(':',1)[1];assert sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[path] and x['earlier_additional_original_frontier_credit']==0;check_chain(path,proof['supporting_paths'][path],b)
assert [x['original_index'] for x in a['earlier_reinspection_sources'][:3]]==[2249,2250,2248] and a['earlier_reinspection_sources'][3]['population']=='later-material' and a['earlier_reinspection_sources'][3]['population_index']==5
litter=current[1564];old=historic[1564];shrub=current[1562];assets=current[1565];batch=current[1566];capture=current[1567];mesh=current[1569]
assert b'5 + (hash.to_u64() % 2) as usize' in old and b'5 + streams::TWIG_SIDES.rng(hash, &[]).index(2)' in litter and b'if streams::TWIG_SIDES.rng(hash, &[]).index(2) == 0' in litter and b'edges.values().all(|count| *count <= 2)' in litter
assert b'ground_scatter_reach(SHRUB_FOOTPRINT_METRES)' in shrub and b'SHRUB_FOOTPRINT_METRES: f32 = 4.5' in shrub and b'height_metres: 2.65' in s['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/geometry.rs'] and b'above: TUFT_HEIGHT_MARGIN_METRES' in s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass.rs']
spec=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/specimens.rs'];env=json.loads(s['content/tactical/garden-hazel-envelope.json']);assert b'GardenSpecimen::CommonHazel.envelope().seed' in spec and b'COMMON_HAZEL_PARAMETERS' in spec and env['max_height_metres']>3 and b'envelope.contains_local_vertex' in spec and b'assert_eq!(hull.len(), envelope.hull_metres.len())' in spec
assert b'GroundCover::TallGrass => leaf_litter_proximity' in litter and b'(WOODLAND_FLOOR_TRANSITION_METRES / ground.grid_scale()).ceil() as isize' in litter and b'for dz in -radius..=radius' in litter and b'for dx in -radius..=radius' in litter and b'grid_width < 2' in s['crates/adventuresim-tactical-core/src/scene/ground.rs'] and b'!scale.is_finite()' in s['crates/adventuresim-tactical-core/src/scene/ground.rs']
assert Fraction(32,10)/Fraction(1,10**19)>2**63-1 and Fraction(12*10**8,1)/Fraction(55,100)>2**31-1
assert b'NonnegativeLength::from_metres(0.55)' in capture and b'(position.x / cell_size).floor() as i32' in capture and b'leaves_by_cell.get(&(cell_x + dx, cell_z + dz))' in capture and b'if maximum_separation == NonnegativeLength::ZERO || maximum_pairs == 0' in capture and b'finite(metres, GeometryRole::Position)?' in s['crates/adventuresim-building-generator/src/spatial_geometry/coordinates.rs']
assert b'0.25..=2_000.0' in s['crates/adventuresim-tactical-core/src/scene_input.rs'] and b'const MAX_PLAYABLE_SIDE: usize = 601' in s['crates/adventuresim-tactical-core/src/scene_input.rs'] and b'let Some(source) = meshes.get(source) else' in batch and b'return Ok(())' in batch and b'Validate every missing family before changing any retained cache entry' in assets
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']);b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for n,indices in [(482,[1562]),(483,[1563,1564]),(484,[1565,1566,1567]),(485,[1568,1569,1570])]:
 receipt=json.loads(records[f'segovb/READ-L12-{n}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False and check['receipt_sha256']==m[f'segovb/READ-L12-{n}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in receipt['rows']:assert row['registration'] in records and row['registration_sha256']==m[row['registration']]['sha256']
adler=json.loads(records['adler/READ-L11-220.json']);check=json.loads(records['adler/VERIFY-L11-220.json']);assert [x['original_index'] for x in adler['sources']]==list(range(1562,1571)) and adler['current_reconciled'] is False and adler['validated'] is False and check['receipt_sha256']==m['adler/READ-L11-220.json']['sha256'];assert [x.get('original_index') for x in adler['connected_reinspection'][:3]]==[2249,2250,2248] and all(x['additional_original_frontier_credit']==0 for x in adler['connected_reinspection'])
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (76,1316,1317,1318,1319));p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==4 and all(sum(y['id']==x['id'] and y['record']=='ASSESSMENT-UNDERSTORY-LITTER-95E.json' for y in ledger)==1 and x['accepted_historical_unique_count_change']==0 for x in a['conditional_findings']) and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical bodies, earlier future/later attribution, current source transition and symbolic bounds; no product execution.')
