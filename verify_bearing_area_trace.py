"""Verify saved source and a rational area illustration; not product execution."""
import gzip,hashlib,json,tarfile
from fractions import Fraction as F
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-BEARING-AREA-95E.json'; doc=json.loads((root/name).read_text()); assert doc['lens']==1 and doc['source_revision']==rev

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  h,p=line.split('\t',1); _,_,oid=h.split(); assert p not in out; out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']] and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); assert s['path'] not in source; source[s['path']]=raw.decode()
assert len(source)==15
p='crates/adventuresim-building-generator/src/'
metric=source[p+'audit/bonding_metrics.rs']; audit=source[p+'audit/projected_defenses.rs']; helpers=source[p+'audit/resolved_geometry_helpers.rs']; gen=source[p+'generator/projected_defenses.rs']; model=source[p+'model/resolved_shapes.rs']; plan=source[p+'model/plan.rs']; network=source[p+'model/resolved_networks.rs']; coords=source[p+'spatial_geometry/coordinates.rs']; orient=source[p+'spatial_geometry/orientation.rs']; cuboid=source['crates/fabelgeist-geometry/src/cuboid.rs']; measured=source['crates/fabelgeist-geometry/src/measurement.rs']; core=source[p+'audit/core.rs']; interface_tests=source[p+'audit/interface_contract_tests.rs']; tests=source[p+'audit/tests.rs']
for x in ('left_extent + right_extent - delta.dot(axis).abs()', 'overlap(local_x, left.size.metres().x * 0.5) * overlap(local_z, left.size.metres().z * 0.5)'): assert x in metric
for x in ('defense.kind == ProjectedDefenseKind::Breteche','support_solids.len() >= 5','plates.len() == 2','plate.size.metres().x * 0.5 - 0.47','support.role != SolidRole::RoofPlate','<= 0.025','resolved_plan_overlap_area(support, plate)?.square_metres()','>= 0.014','resolved_solid_contains_point(','0.12,','unsupported_projected_defense_roof'): assert x in audit
assert 'solid.size.metres() * 0.5 + Vec3::splat(tolerance)' in helpers and 'local.abs().cmple(half).all()' in helpers
for x in ('let length = run.start.distance(run.end);','let post_plan = inner_plate_plan + tangent * side * (length * 0.5 - 0.38);','Vec3::new(0.18, post_height, 0.18)','Vec3::new(length + 0.18, plate_height, 0.18)','SolidRole::RoofPlate'): assert x in gen
assert 'pub centre: crate::spatial_geometry::Position<Architectural>' in model and 'pub size: crate::spatial_geometry::CuboidDimensions' in model and 'pub resolved_geometry: ResolvedGeometry' in plan and 'pub solids: Vec<ResolvedSolid>' in network
assert 'finite(metres, GeometryRole::Position)?' in coords and 'pub const ZERO: Self = Self(0.0);' in orient and 'if !component.is_finite()' in cuboid and 'if component.metres() < 0.0' in cuboid and 'if !value.is_finite()' in measured and 'if value < 0.0' in measured
assert 'audit_projected_defenses(plan, issues)?' in core and 'resolved_plan_overlap_area(&left, &right)' in interface_tests and 'native_geometry.x += 0.55;' in tests
h=doc['historical_source']; old=json.loads((root/h['ledger']).read_text())
def find(x):
 if isinstance(x,dict):
  if x.get('contract_id')==h['contract_id']: yield x
  for v in x.values(): yield from find(v)
 elif isinstance(x,list):
  for v in x: yield from find(v)
records=list(find(old)); assert len(records)==1 and 'no new retained unique' in records[0]['status'] and 'actual consumers/admitted orientations' in records[0]['first_failed_contract']
r=h['source_role']; role=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles'][r['index']]; assert role['source_id']==r['source_id'] and role['historical_sha256']==r['sha256']==hashlib.sha256(metric.encode()).hexdigest()
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as t: assert hashlib.sha256(t.extractfile(r['sha256']).read()).hexdigest()==r['sha256']
# Exact arithmetic illustration with ideal axis-aligned intervals; no Rust or geometry runtime.
L=F(2); plate_half=(L+F(18,100))/2; post_half=F(9,100); x=L/2-F(38,100); z=F(13,100); span_x=post_half+plate_half-x; span_z=post_half+F(9,100)-z; predicted=span_x*span_z
actual_x=min(x+post_half,plate_half)-max(x-post_half,-plate_half); actual_z=min(z+post_half,F(9,100))-max(z-post_half,-F(9,100)); actual=actual_x*actual_z
assert x==F(62,100) and span_x==F(56,100) and span_z==F(5,100) and predicted==F(28,1000) and actual==F(9,1000) and actual<F(14,1000)<=predicted and z<post_half+F(12,100)
c=doc['candidates'][0]; entries=json.loads((root/'BASELINE-95E-20261110.json').read_text())['newly_assessed_ledger']['entries']; progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert sum(x['id']==c['id'] and x['record']==name for x in entries)==1 and progress['new_current_ledger_candidates']==len(entries) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified fifteen Git sources, historical metric role and rational bearing-area illustration')
