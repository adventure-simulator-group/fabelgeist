"""Verify preserved grass source and a rational illustration, not product behavior."""
import gzip,hashlib,json,tarfile,tomllib
from fractions import Fraction as F
from pathlib import Path
root=Path(__file__).resolve().parent
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-GRASS-MESH-95E.json'
doc=json.loads((root/name).read_text())
base=json.loads((root/'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  head,p=line.split('\t',1); _,_,oid=head.split(); assert p not in out; out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
 assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]
 assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==11
prefix='crates/adventuresim-tactical-client/src/presentation/'
grass=source[prefix+'ground_scatter/grass.rs']; placement=source[prefix+'ground_scatter/instanced_grass.rs']; batches=source[prefix+'ground_scatter/instanced_grass/batches.rs']; material=source[prefix+'ground_scatter/instanced_grass/material.rs']; pipeline=source[prefix+'chunk_instancing/pipeline.rs']; abi=source[prefix+'chunk_instancing/chunk_instancing.wgsl']; shader=source['assets/shaders/tactical_grass_instanced.wgsl']; foliage=source['assets/shaders/tactical_foliage.wgsl']; config=source['assets/config/tactical-graphics.yaml']
for marker in ('pub(in crate::presentation) fn grass_tuft_mesh(', 'grass_ribbon_patch_mesh_with_rows(', 'if lod == GrassMeshLod::Near', '&& branch_count > 0', 'inclusive_unit_f32() < 0.125', 'let stem_side = Vec3::new(angle.cos(), 0.0, angle.sin()) * 0.0025;', 'start - stem_side,', 'start + stem_side,', 'end - stem_side * 0.62,', 'end + stem_side * 0.62,', 'uvs.extend_from_slice(&[[attachment_height, -attachment_fraction]; 4]);', 'normals.extend_from_slice(&[normal; 4]);', 'blade_roots.extend_from_slice(&[blade_root; 4]);', 'colors.extend_from_slice(&[color; 4]);', 'mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, blade_roots);', 'mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);'): assert marker in grass
for marker in ('GrassMeshLod::Near,', 'ChunkMaterialPlugin::<TacticalGrassInstancedMaterial>::default()', 'spawn_tuft_batches('): assert marker in placement
for marker in ('if instances.is_empty()', 'grass_tuft_mesh(', 'material: material.clone()'): assert marker in batches
assert 'shaders/tactical_grass_instanced.wgsl' in material and 'GRASS_INSTANCED_SHADER.into()' in material
for marker in ('self.mesh_pipeline.specialize(key.mesh_key, layout)?', 'descriptor.vertex.shader = self.shader.clone()'): assert marker in pipeline
for marker in ('#ifdef VERTEX_UVS_B', '@location(3) uv_b: vec2<f32>', '#ifdef VERTEX_COLORS', '@location(5) color: vec4<f32>'): assert marker in abi
for marker in ('let bend = clamp(vertex.uv.y, 0.0, 1.0);', 'let half_width = length(collapsed_position.xz - blade_root_local.xz)', 'let signed_side = select(-1.0, 1.0, vertex.uv.x >= 0.5);', 'let side_offset = visible_side * half_width * signed_side;'): assert marker in shader
assert 'let is_inflorescence = vertex.uv.y < 0.0;' in foliage and 'if is_inflorescence {' in foliage and 'grass:\n  enabled: true' in config
m=json.loads((root/'native-evidence/bevy_pbr-0.19.1/MANIFEST.json').read_text()); archive=(root/'native-evidence/bevy_pbr-0.19.1'/m['archive']).read_bytes(); lock=tomllib.loads(source['Cargo.lock']); item=next(x for x in lock['package'] if x['name']=='bevy_pbr' and x['version']=='0.19.1'); assert hashlib.sha256(archive).hexdigest()==m['sha256']==item['checksum']
with tarfile.open(root/'native-evidence/bevy_pbr-0.19.1'/m['archive']) as t:
 raw=t.extractfile('bevy_pbr-0.19.1/src/render/mesh.rs').read(); entry=next(x for x in m['inspected_members'] if x['path'].endswith('/src/render/mesh.rs')); assert len(raw)==entry['bytes'] and hashlib.sha256(raw).hexdigest()==entry['sha256']; mesh=raw.decode()
for marker in ('if layout.0.contains(Mesh::ATTRIBUTE_UV_1)', 'shader_defs.push("VERTEX_UVS_B".into())', 'Mesh::ATTRIBUTE_UV_1.at_shader_location(3)', 'if layout.0.contains(Mesh::ATTRIBUTE_COLOR)', 'shader_defs.push("VERTEX_COLORS".into())', 'Mesh::ATTRIBUTE_COLOR.at_shader_location(5)'): assert marker in mesh
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text())
def find(x):
 if isinstance(x,dict):
  if x.get('id')==h['contract_id']: yield x
  for value in x.values(): yield from find(value)
 elif isinstance(x,list):
  for value in x: yield from find(value)
old=list(find(ledger)); assert len(old)==1 and '+0 unique' in old[0]['status'] and 'No ordinary supported producer has yet been assessed' in old[0]['first_failed_contract']
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as t:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(t.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; path=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[path].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
# Rational source-form illustration, not evaluation of Rust, WGSL or GPU arithmetic.
root_x=F(1,4); lower=F(1,400); upper=lower*F(62,100)
assert abs((root_x-lower)-root_x)==abs((root_x+lower)-root_x)==lower
assert abs((root_x-upper)-root_x)==abs((root_x+upper)-root_x)==upper
vertices=[(lower,F(1)),(lower,F(1)),(upper,F(2)),(upper,F(2))]
def area(a,b,c): return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
assert area(vertices[0],vertices[1],vertices[3])==area(vertices[0],vertices[3],vertices[2])==0
c=doc['candidates'][0]; entries=base['newly_assessed_ledger']['entries']; found=[x for x in entries if x['id']==c['id']]; assert len(found)==1 and found[0]['record']==name and progress['new_current_ledger_candidates']==len(entries) and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified eleven Git sources, lock-bound Bevy layout, historical role bytes and rational source-form illustration')
