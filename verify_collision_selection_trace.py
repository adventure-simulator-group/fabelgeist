"""Verify saved source and historical identity; not collision execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-COLLISION-SELECTION-95E.json'; doc=json.loads((root/name).read_text()); assert doc['lens']==1 and doc['source_revision']==rev

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  h,p=line.split('\t',1); _,_,oid=h.split(); assert p not in out; out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']] and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); assert s['path'] not in source; source[s['path']]=raw.decode()
assert len(source)==11
p='crates/adventuresim-building-generator/src/'; collision=source[p+'collision.rs']; wall_audit=source[p+'audit/wall_openings.rs']; audit=source[p+'audit/core.rs']; generator=source[p+'generator/core.rs']; wall=source[p+'model/walls.rs']; plan=source[p+'model/plan.rs']; resolved=source[p+'model/resolved_networks.rs']; windows=source[p+'windows.rs']; recipes=source['crates/adventuresim-tactical-core/src/scene_input/recipes.rs']; adapter=source['crates/adventuresim-tactical-core/src/scene_input/buildings/collision.rs']; server=source['crates/adventuresim-tactical-server/src/openings.rs']
start=collision.index('pub fn compile_building_collision('); end=collision.index('pub(crate) fn collision_parts(',start); body=collision[start:end]
for x in ('solid.id, solid','wall.replaced_by_owner.is_none()','wall.host_solids.iter().copied()','.filter_map(|id| solids.get(&id).copied())','.map(|solid| collision_parts(plan, solid))','for bar in compile_window_bars(plan)?','Ok(BuildingCollision { bounds, cuboids })'): assert x in body
assert 'audit_plan(' not in body
for x in ('wall.host_solids.is_empty()', 'solids.get(id).is_none_or(|solid|', 'invalid_wall_host_union'): assert x in wall_audit
assert 'audit_wall_opening_assemblies(plan, issues)?;' in audit and 'pub fn audit_plan(' in audit
assert 'pub fn generate(program: &BuildingProgram)' in generator and 'generate_unchecked(program, &[])' in generator and 'validate_generated_plan(plan)' in generator and 'let issues = crate::audit_plan(&plan)?;' in generator
assert 'pub host_solids: Vec<ResolvedItemId>' in wall and 'pub replaced_by_owner: Option<GeometryOwnerId>' in wall and 'pub resolved_geometry: ResolvedGeometry' in plan and 'pub solids: Vec<ResolvedSolid>' in resolved
assert 'pub fn compile_window_bars(' in windows and 'let source = ResolvedItemId((7_u64 << 60)' in windows
assert 'let plan = generate(&program)?;' in recipes and 'let collision = compile_building_collision(&plan)?;' in recipes
for x in ('collision\n            .cuboids','Collider::compound(', 'Collider::cuboid('): assert x in adapter
for x in ('let plan = generate_building(&building.program)?;', 'let collision = compile_building_collision(&plan)?;', 'RigidBody::Static', 'compile_tactical_building_collider(&collision)?'): assert x in server
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text())
def find(x):
 if isinstance(x,dict):
  if x.get('contract_id')==h['contract_id']: yield x
  for v in x.values(): yield from find(v)
 elif isinstance(x,list):
  for v in x: yield from find(v)
old=list(find(ledger)); assert len(old)==1 and 'nonexistent selected ID drop' in old[0]['first_failed_contract'] and 'currently conditional' in old[0]['first_failed_contract']
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as t:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(t.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; path=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[path].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
c=doc['candidates'][0]; entries=json.loads((root/'BASELINE-95E-20261110.json').read_text())['newly_assessed_ledger']['entries']; progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert sum(x['id']==c['id'] and x['record']==name for x in entries)==1 and progress['new_current_ledger_candidates']==len(entries) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified eleven Git sources, collision selection and qualified historical records')
