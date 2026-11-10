"""Verify heating-frame evidence and ideal rectangle arithmetic without running product code."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from fractions import Fraction as F
from pathlib import Path
root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-HEATING-FRAME-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision
def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1); _, _, oid = header.split(); assert name not in out; out[name] = oid
    return out
current = tree('main-advance-95e/current-tree.tsv'); previous = tree('main-advance-95e/previous-tree.tsv'); sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip' and item['revision'] == revision
    raw = gzip.decompress(path.read_bytes()); oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    assert all(1 <= a <= b <= item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 16
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
placement = sources[p+'heating/placement.rs']
must(placement, ['const CORE_WIDTH_METRES: f32 = 0.96;', 'const CORE_HALF_DEPTH_METRES: f32 = 0.9;', 'Self::Compact => CORE_HALF_DEPTH_METRES', 'let tangent = Vec2::new(-axis.y, axis.x);', 'let a = point(min);', 'let b = point(max);', 'a.min(b)', 'a.max(b)', 'TIMBER_CLEARANCE_METRES: f32 = 0.08', '.clear(plan, self.site.body()?, HostWallPolicy::Replace)?', 'PlanDirection::from_normalized(-wall.frame.outward)?', 'PlanDirection::from_normalized(wall.frame.outward)?'])
must(sources[p+'spatial_geometry/orientation.rs'], ['pub fn from_normalized(value: Vec2)', 'Dir2::new(value)', 'if !value.is_normalized()', 'Dir2::new_unchecked(value)'])
must(sources[p+'spatial_geometry/coordinates.rs'], ['pub struct Displacement', 'finite(metres, GeometryRole::Displacement)?'])
must(sources[p+'spatial_geometry/bounds.rs'], ['if lower > upper', 'Self::new(Position::from_metres(min)?, Position::from_metres(max)?)'])
must(sources[p+'model/walls.rs'], ['pub struct WallLocalFrame', 'pub tangent: Vec2', 'pub outward: Vec2'])
must(sources[p+'heating.rs'], ['placement::find(plan)?', 'partition::cut(plan, placement)?', 'appliances::build(&mut assembly, top)?'])
must(sources[p+'heating/assembly.rs'], ['let bounds = self.placement.site.bounds(min, max)?;', 'RigidRotation::IDENTITY', 'bounds: self.placement.site.bounds(min, max)?'])
must(sources[p+'heating/partition.rs'], ['let cut = placement.site.bounds(', 'subtract(source.cuboid_bounds()?, cut)?'])
must(sources[p+'heating/appliances.rs'], ['Part::Footing', '-CORE_HALF_WIDTH_METRES', '-CORE_HALF_DEPTH_METRES'])
must(sources[p+'heating/model.rs'], ['pub kitchen_axis: crate::spatial_geometry::PlanDirection', 'pub operating_space: SpatialBounds<Architectural>'])
must(sources[p+'generator/wall_assemblies.rs'], ['let outward = direction_vector(wall.direction);', 'let source = crate::WallSourceId::StoreyWall {'])
must(sources[p+'generator/defense_geometry.rs'], ['Direction::North => Vec2::Y', 'Direction::East => Vec2::X', 'Direction::South => -Vec2::Y', 'Direction::West => -Vec2::X'])
must(sources[p+'generator/orchestration.rs'], ['crate::heating::resolve(program, &mut plan)?'])
must(sources[p+'solid_overlap.rs'], ['pub(crate) fn overlaps_bounds(', 'solid_radius + bounds_radius - delta.dot(axis).abs() > tolerance'])
lock = tomllib.loads(sources['Cargo.lock']); packages = {(x['name'],x['version']): x for x in lock['package']}
assert 'glam 0.32.1' in packages[('bevy_math','0.19.1')]['dependencies']
contents = {}
for dependency in doc['native_dependencies']:
    manifest_path = (root / dependency['manifest']).resolve(); assert manifest_path.is_relative_to(root)
    manifest = json.loads(manifest_path.read_text()); archive = manifest_path.parent / manifest['archive']; raw = archive.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == dependency['archive_sha256'] == manifest['sha256'] == packages[(manifest['package'],manifest['version'])]['checksum']
    with tarfile.open(archive,'r:gz') as tar:
        entry = next(x for x in manifest['inspected_members'] if x['path'] == dependency['source_member'])
        member = tar.extractfile(entry['path']).read(); assert hashlib.sha256(member).hexdigest() == entry['sha256'] and len(member) == entry['bytes']
        contents[manifest['package']] = member.decode()
must(contents['bevy_math'], ['pub fn new(value: Vec2)', 'pub fn new_and_length(value: Vec2)', 'length.is_finite() && length > 0.0', 'fn assert_is_normalized'])
must(contents['glam'], ['pub fn is_normalized(self) -> bool', 'math::abs(self.length_squared() - 1.0) <= 2e-4'])
# Ideal rational arithmetic; this does not execute product f32 or constructors.
axis=(F(3,5),F(4,5)); tangent=(-axis[1],axis[0]); hx=F(12,25); hz=F(9,10)
transform=lambda x,z:(tangent[0]*x+axis[0]*z, tangent[1]*x+axis[1]*z)
a=transform(-hx,-hz); b=transform(hx,hz)
assert a[0] == -F(39,250) and b[0] == F(39,250)
actual=abs(tangent[0])*hx+abs(axis[0])*hz
assert actual == F(231,250)
point=transform(-F(9,20),F(4,5)); assert point == (F(21,25),F(37,100)) and point[0] > b[0]+F(2,25)
h = doc['historical_source']; ledger = json.loads((root / h['ledger']).read_text())
def find(x):
    if isinstance(x,dict):
        if x.get('contract_id') == h['contract_id']: yield x
        for v in x.values(): yield from find(v)
    elif isinstance(x,list):
        for v in x: yield from find(v)
records=list(find(ledger)); assert len(records)==1 and 'cardinal restriction' in records[0]['first_failed_contract']
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as archive:
    for item in h['historical_source_roles']:
        r=roles[item['index']]; path=item['source_id'].split(':',1)[1]
        assert r['source_id']==item['source_id'] and r['historical_sha256']==item['sha256']==hashes[item['index']]
        assert r['historical_assessment']==item['assessment'] and hashlib.sha256(archive.extractfile(item['sha256']).read()).hexdigest()==item['sha256']
        assert hashlib.sha256(sources[path].encode()).hexdigest()==item['sha256']
baseline=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); candidate=doc['candidates'][0]; entries=baseline['newly_assessed_ledger']['entries']
assert sum(x['id']==candidate['id'] and x['record']==name for x in entries)==1 and progress['new_current_ledger_candidates']==len(entries)
assert candidate['discovery'].startswith('partial') and candidate['repair']=='not started' and candidate['validation']=='not performed'
print('verified 16 Git sources, two lock-bound archives, historical roles and ideal geometry; no product ran')
