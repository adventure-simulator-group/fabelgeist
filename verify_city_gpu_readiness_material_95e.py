#!/usr/bin/env python3
"""Verify source and original-attribution evidence, not product behavior."""
import gzip,hashlib,json,tarfile,tomllib
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-CITY-GPU-READINESS-MATERIAL-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1511-1520.json');w=load('COVERAGE-L01-ROLES.json');proof=load('CITY-GPU-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1511,1521)) and [x['lines'] for x in h['historical_source_roles']]==[50,134,449,430,103,151,102,123,139,107]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1511-1520.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
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
props=current[1511];ranges=current[1512];compute=current[1514];material=current[1517];mod=current[1518];scratch=current[1519]
furniture=s['crates/adventuresim-tactical-client/src/presentation/furniture.rs'];input_=s['crates/adventuresim-tactical-client/src/presentation/buildings/gpu/assembly/input.rs'];assembly=s['crates/adventuresim-tactical-client/src/presentation/buildings/gpu/assembly.rs'];materials=s['crates/adventuresim-tactical-client/src/presentation/buildings/materials.rs'];palette=s['crates/adventuresim-tactical-client/src/presentation/buildings/materials/palette.rs'];shader=s['assets/shaders/tactical_city.wgsl'];handles=s['crates/adventuresim-procedural-textures/src/runtime/handles.rs'];plaster=s['crates/adventuresim-procedural-textures/src/lime_plaster.rs'];bake=s['crates/adventuresim-procedural-textures/src/bake.rs']
assert b'transform: transform.to_matrix()' in props and b'local_transform: Mat4::IDENTITY' in props and b'placements: vec![part.transform]' in input_ and b'Transform::IDENTITY' in furniture and b'.after(bevy::transform::TransformSystems::Propagate)' in furniture and b'.add_systems(Last, assembly::assemble)' in mod
assert b'owners.sort_unstable()' in ranges and b'&[0, 0, 1, 2, 2]' in ranges and b'(owner_start + length).checked_mul(clusters)' in ranges
assert scratch.count(b'return false;')>=4 and b'max_storage_buffer_binding_size' in scratch and b'let mut prepared = Vec::new()' in scratch and scratch.index(b'self.batches = prepared;')>scratch.index(b'max_storage_buffer_binding_size')
assert compute.index(b'prepared.slots.clear();')<compute.index(b'if !scratch.prepare(') and b'READY.get(owner).store(true, Ordering::Relaxed)' in compute and b'READY.get(owner).store(false' in assembly
assert b'palette.get(batch.material)' in input_ and b'CityMaterial::from_source(authored)' in assembly and b'WallMaterialClass::TimberInfill) => palette.infill.clone()' in palette
assert b'&textures.lime_plaster' in materials and b'metallic_roughness_texture: Some(textures.arm.clone())' in materials and b'occlusion_texture: Some(textures.arm.clone())' in materials and b'metallic: 1.0' in materials and b'Affine2::from_scale' in materials
assert b'pub albedo:' in material and b'pub normal:' in material and b'metallic_roughness_texture' not in material and b'occlusion_texture' not in material and b'source.perceptual_roughness' in material and b'source.metallic' in material
assert b'pbr.material.perceptual_roughness = surface.properties.x;' in shader and b'pbr.material.metallic = surface.properties.y;' in shader and b'textureSample(normal_map' in shader
assert b'lime_plaster: SurfaceTextureSet::load(residency, images, LimePlaster)' in handles and b'arm: load_map(residency, images, recipe, MapChannel::Arm)' in handles and b'LimePlaster => crate::generate_lime_plaster_textures(params, images)' in bake and b'arm.extend_from_slice(&[encode_unit(sample.ao), encode_unit(roughness), 0, 255])' in plaster
native=load(a['native_source_evidence']['archive_manifest']);archive=(r/'native-evidence/bevy_pbr-0.19.1'/native['archive']).read_bytes();lock=tomllib.loads(s['Cargo.lock'].decode());pkg=next(x for x in lock['package'] if x['name']=='bevy_pbr' and x['version']=='0.19.1');assert len(archive)==native['bytes'] and sha(archive)==native['sha256']==pkg['checksum']
with tarfile.open(r/'native-evidence/bevy_pbr-0.19.1'/native['archive'],'r:gz') as t:
 for path in a['native_source_evidence']['inspected_ranges']:
  b=t.extractfile(path).read();info=next(x for x in native['inspected_members'] if x['path']==path);assert sha(b)==info['sha256'] and len(b)==info['bytes']
  assert all(1<=lo<=hi<=len(b.splitlines()) for lo,hi in a['native_source_evidence']['inspected_ranges'][path])
  if path.endswith('pbr_material.rs'):native_material=b
  else:native_shader=b
assert b'metallic *= metallic_roughness.b;' in native_shader and b'perceptual_roughness *= metallic_roughness.g;' in native_shader and b'diffuse_occlusion *=' in native_shader and b'occlusion_texture' in native_shader
assert b'Those values are multiplied by the scalar ones' in native_material
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(460,[1511,1512,1513]),(461,[1514]),(462,[1515,1516,1517,1518]),(463,[1519,1520]),(464,[1521])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
for batch in (214,215,231,242,244,245):
 receipt=json.loads(records[f'adler/READ-L11-{batch}.json']);check=json.loads(records[f'adler/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert check['receipt_sha256']==m[f'adler/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{batch}.json']['sha256']
assert [x['original_index'] for x in json.loads(records['adler/READ-L11-215.json'])['sources']]==[1520,1521,1522,1523]
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1294,1295,1296,1297));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1294,1295,1296,1297)]==[1511,1514,1517,1519]
old_findings=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==2 and all(c['id'] in old_findings and sum(x['id']==c['id'] and x['record']=='ASSESSMENT-CITY-GPU-READINESS-MATERIAL-95E.json' for x in ledger)==1 and c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified ten historical GPU bodies, earlier attribution, and bounded readiness and material source connections; no product execution.')
