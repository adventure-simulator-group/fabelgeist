#!/usr/bin/env python3
"""Verify restored source/provenance and symbolic bounds, not product behavior."""
import gzip,hashlib,json,tarfile
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1549-1553.json');w=load('COVERAGE-L01-ROLES.json');proof=load('GROUND-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
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
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1549,1554)) and [x['lines'] for x in h['historical_source_roles']]==[37,1237,263,70,32]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1549-1553.json';assert x['new_personal_read_ranges']==([[1,420],[421,840],[841,1237]] if i==1550 else [[1,x['lines']]])
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur;check_chain(c['path'],proof['paths'][c['path']],raw)
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
ground=s['crates/adventuresim-tactical-core/src/scene/ground.rs'];urban=s['crates/adventuresim-tactical-core/src/scene/ground/urban.rs'];scene=s['crates/adventuresim-tactical-core/src/scene.rs'];admission=s['crates/adventuresim-tactical-core/src/scene/admission.rs'];input_=s['crates/adventuresim-tactical-core/src/scene_input.rs'];validation=s['crates/adventuresim-tactical-core/src/scene_input/validation.rs'];build=s['crates/adventuresim-tactical-core/src/scene_ground.rs'];raster=s['crates/adventuresim-tactical-client/src/presentation/terrain/ground_map.rs'];city=s['crates/adventuresim-tactical-client/src/presentation/generation/city.rs'];worker=s['crates/adventuresim-tactical-client/src/presentation/generation/worker_products.rs'];focused=s['crates/adventuresim-tactical-client/src/presentation/vista/focused.rs'];material=s['crates/adventuresim-tactical-client/src/presentation/terrain/material.rs'];net=s['crates/adventuresim-tactical-netcode/src/replication.rs'];config=s['crates/adventuresim-tactical-client/src/presentation/config.rs'];inst=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass.rs'];grass=current[1550];mask=current[1549]
assert b'#[derive(Component, Serialize, Deserialize, Debug, Reflect, Clone, PartialEq)]' in ground and b'self.samples.len() / self.width' in ground and b'grid_width < 2' in ground and b'!scale.is_finite()' in ground and b'samples.len() != grid_width.checked_mul(grid_depth)?' in ground
assert b'pub enum GroundCover' in ground and b'#[default]\n    TallGrass' in ground and b'UrbanGroundSurfaces::default()' in ground and b'#[derive(Default, Serialize, Deserialize, Debug, Reflect, Clone)]' in urban
assert b'pub fn from_heightmap(' in scene and b'admission::validate_dimensions' in scene and b'!scale.is_finite()' in admission and b'scale <= 0.0' in admission and b'0.25..=2_000.0' in input_ and b'validate_grid(&self.playable' in validation and b'SceneGround::from_samples(width, depth, spacing, samples)' in build
assert all(x in raster for x in (b'const GROUND_PRESENTATION_SAMPLES_PER_CELL: usize = 6;',b'let width = (source_width - 1) * GROUND_PRESENTATION_SAMPLES_PER_CELL + 1;',b'encode_canopy_floor_distance(ground, width, depth, &mut pixels);',b'let outer_radius = (4.8 / metres_per_pixel).ceil().max(1.0) as usize;',b'.map(|index| if sources(index) { 0 } else { maximum + 1 })',b'ground.grid_depth()')) and b'grass_cover_mask_pixels(ground, seed)?' in mask
assert b'pub surface_ground:' in city and b'if city.matches(&input, &graphics)' in worker and b'Some(&product.surface_ground)' in focused and b'ground_map_image(' in material and b'.replicate::<SceneGround>()' in net
assert b'tier.native_tufts_per_cell_side == 0 || tier.native_blades_per_tuft_side == 0' in config and b'grid_side * grid_side' in grass and b'positions.len() as u32' in grass and b'grass.lod.near.native_tufts_per_cell_side' in inst and b'} as i32;' in inst
native=a['native_evidence'][0];doc=contents(native['file']);assert sha(doc)==native['sha256'] and b'Values larger than the maximum integer value' in doc and b'will saturate to the maximum value of the integer type' in doc and b'Integer operators will panic when they overflow when compiled in debug mode' in doc
# Symbolic conservative bound: the finite 1e-20 scale divided by six is below
# 2e-20; even 4.8 divided by this looser upper bound exceeds u64::MAX.
assert Fraction(48,10)/Fraction(2,10**20)>2**64-1 and Fraction(2,10**20)<Fraction(1,4)
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']);b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(477,[1550]),(478,[1551,1552,1553])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False and check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
 for row in receipt['rows']:assert row['registration'] in records and row['registration_sha256']==m[row['registration']]['sha256']
partial=[json.loads(records[f'segovb/PARTIAL-L12-477-{name}.json']) for name in ('001','002','003-EOF')];assert [x['fresh_personal_body_reads'] for x in partial]==[[[1,600]],[[601,1200]],[[1201,1237]]] and [x['unread_tail'] for x in partial]==[[[601,1237]],[[1201,1237]],[]] and all(x['current_source_reconciled'] is False and x['original_credit']==0 for x in partial)
for n in (219,290,291,299):
 receipt=json.loads(records[f'adler/READ-L11-{n}.json']);check=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False and check['receipt_sha256']==m[f'adler/READ-L11-{n}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{n}.json']['sha256']
assert any(x.get('original_index')==1901 and x['fresh_reads']==[[1,182]] for x in json.loads(records['adler/READ-L11-290.json'])['sources'])
prior=load('ASSESSMENT-GENERATION-WORKER-TRUST-95E.json');refs={x['original_path']:x for x in prior['recovered_historical_records']};earlier=json.loads(contents(refs['segovb/READ-L12-476.json']['preserved_bytes']));assert any(x['original_index']==1549 for x in earlier['rows']) and earlier['current_source_reconciled'] is False
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (110,1303,1310,1311,1312));p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert [x['id'] for x in a['conditional_findings']]==['ADLER-L11-SCENE-GROUND-WIRE-ADMISSION','REVIEW-GROUND-RASTER-CHAMFER-CAPACITY-001'] and all(sum(y['id']==x['id'] and y['record']=='ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json' for y in ledger)==1 for x in a['conditional_findings']) and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical grass bodies, partial-read attribution, pinned ground sources and symbolic capacity bound; no product execution.')
