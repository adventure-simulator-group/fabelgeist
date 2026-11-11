#!/usr/bin/env python3
"""Verify old/current grass source, attributed receipts and static arithmetic."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1554-1561.json');w=load('COVERAGE-L01-ROLES.json');proof=load('GRASS-CHUNK-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
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
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1554,1562)) and [x['lines'] for x in h['historical_source_roles']]==[623,86,54,47,88,7,24,167]
current={};historic={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1554-1561.json' and x['new_personal_read_ranges']==([[1,325],[326,623]] if i==1554 else [[1,x['lines']]])
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historic[i]=raw;c=x['current_comparison'];check_chain(c['path'],proof['paths'][c['path']],raw)
  if i in (1556,1561):assert c['status']=='absent in verified current Git tree' and c['path'] not in tree and c['byte_identical'] is False
  else:
   cur=contents(c['preserved_bytes']);assert c['status']=='present' and c['source_revision']==rev and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']] and c['byte_identical']==(raw==cur);current[i]=cur
   if i in (1554,1555,1557,1558):
    diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode();d=c['exact_diff'];assert diff==contents(d['preserved_bytes']) and sha(diff)==d['sha256'] and len(diff)==d['bytes']
   else:assert i in (1559,1560) and raw==cur
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
cfg=s['crates/adventuresim-tactical-client/src/presentation/config.rs'];grass=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/grass.rs'];ground=s['crates/adventuresim-tactical-client/src/presentation/terrain/ground_map.rs'];spawn=s['crates/adventuresim-tactical-client/src/presentation/chunk_instancing/spawn.rs'];queue=s['crates/adventuresim-tactical-client/src/presentation/chunk_instancing/queue.rs'];shader=s['assets/shaders/tactical_grass_instanced.wgsl'];yaml=s['assets/config/tactical-graphics.yaml'];lock=s['Cargo.lock'];batch=current[1555];inst=current[1554];diag=current[1557]
assert b'pub cover_mask_feather_m: f32' in cfg and b'positive(self.grass.transition.cover_mask_feather_m)' in cfg and b'cover_mask_feather_m: 4.8' in yaml and b'grass_cover_mask_pixels(' in ground and b'(4.8 / metres_per_pixel)' in ground and b'cover_mask_feather_m' not in ground
assert all(x in cfg for x in (b'pub root_stratum_side: Option<usize>',b'pub width_compensation: Option<f32>',b'pub width_compensation_limit: Option<f32>')) and b'root_stratum_side' not in grass and b'GRASS_FAR_STRATUM_SIDE: usize = 4' in grass and b'GRASS_VISTA_STRATUM_SIDE: usize = 8' in grass and b'.sqrt().min(1.65)' in grass and b'lod.width_compensation(grass_density)' in diag
assert b'grass.placement.playable_patch_spacing_m' in grass and b'tier.native_tufts_per_cell_side as f32' in grass and b'playable_patch_spacing_m: 1.81' in yaml and b'native_tufts_per_cell_side: 12' in yaml and b'minimum_push: 0.7' in yaml and b'maximum_push: 1.35' in yaml
assert b'horizontal: footprint' in inst and b'above: TUFT_HEIGHT_MARGIN_METRES' in inst and b'below: TUFT_DEPTH_MARGIN_METRES' in inst and b'ground_scatter_reach(configured_tuft_footprint_metres(lod, grass))' in batch and b'NoAutoAabb' in spawn and b'NoFrustumCulling' not in spawn and b'reach.horizontal * largest_scale' in spawn and b'entities_cpu_culling' in queue
assert b'push_direction * 0.62' in shader and b'grass.interaction_motion.w' in shader and b'let curve_offset = total_curve * curve_profile * blade_visibility' in shader and b'wind_offset' in shader and b'curve_offset.x + side_offset.x' in shader
assert Fraction(181,100)/12<Fraction(62,100)*Fraction(7,10) and Fraction(62,100)*Fraction(7,10)==Fraction(217,500)
manifest=load('native-evidence/bevy_camera-0.19.1/MANIFEST.json');assert manifest['sha256'].encode() in lock;ref=a['native_evidence'][0];entry=next(x for x in manifest['inspected_members'] if x['path']==ref['member']);assert entry['sha256']==ref['sha256'] and manifest['personally_inspected_ranges'][ref['member']]==[[748,855]]
with tarfile.open(r/'native-evidence/bevy_camera-0.19.1'/manifest['archive'],'r:gz') as ar:
 b=ar.extractfile(ref['member']).read();assert sha(b)==entry['sha256'] and len(b)==entry['bytes'] and b'if !no_frustum_culling && !no_cpu_culling_camera' in b and b'frustum.intersects_obb(model_aabb' in b
for x in a['search_evidence']:
 b=contents(x['preserved_bytes']);assert sha(b)==x['sha256'] and len(b)==x['bytes'] and x['source_revision']==rev and x['command'][0:3]==['git','grep','-n'] and rev.encode() in b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']);b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for n,indices in [(479,[1554]),(480,[1555,1556,1557,1558]),(481,[1559,1560,1561])]:
 receipt=json.loads(records[f'segovb/READ-L12-{n}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False and check['receipt_sha256']==m[f'segovb/READ-L12-{n}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in receipt['rows']:assert row['registration'] in records and row['registration_sha256']==m[row['registration']]['sha256']
prior=load('ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json');refs={x['original_path']:x for x in prior['recovered_historical_records']};earlier=json.loads(contents(refs['adler/READ-L11-219.json']['preserved_bytes']));assert [x['original_index'] for x in earlier['sources'] if 1554<=x.get('original_index',0)<=1561]==list(range(1554,1562)) and earlier['current_reconciled'] is False and earlier['validated'] is False
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1310,1311,1312,1313,1314,1315));p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==4 and all(sum(y['id']==x['id'] and y['record']=='ASSESSMENT-GRASS-CHUNK-AND-CONFIG-95E.json' for y in ledger)==1 and x['accepted_historical_unique_count_change']==0 for x in a['conditional_findings']) and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified eight historical grass bodies, exact diffs/current absence, attributed receipts and bounded configuration/culling sources; no product execution.')
