"""Verify preserved source for two bounded furniture nonfindings, without product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-FURNITURE-SAFEGUARDS-95E.json'
doc = json.loads((root / name).read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if doc['source_revision'] != revision or doc['lens'] != 1 or name not in progress['new_nonfinding_assessments']:
    raise ValueError('wrong current source attribution')

def tree(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in result:
            raise ValueError('duplicate tree entry')
        result[path] = oid
    return result

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe source reference')
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
        hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + item['path'])
    if item['path'] in sources or any(not 1 <= a <= b <= item['lines'] for a, b in item['ranges']):
        raise ValueError('duplicate source or invalid read range')
    sources[item['path']] = raw.decode()
if len(sources) != 13:
    raise ValueError('unexpected source count')

def src(suffix):
    matches = [v for k, v in sources.items() if k.endswith(suffix)]
    if len(matches) != 1:
        raise ValueError('missing or ambiguous source ' + suffix)
    return matches[0]

def must(text, markers):
    for marker in markers:
        if marker not in text:
            raise ValueError('missing source marker: ' + marker)

turned = src('adventuresim-building-generator/src/furniture/turned.rs')
must(turned, ['const SIDES: usize = 12;', '(0, -Vec3::Y), (profile.len() - 1, Vec3::Y)', 'self.mesh(material).push_triangle('])
lod = src('adventuresim-building-generator/src/lod.rs')
triangle = lod.split('pub(crate) fn push_triangle(', 1)[1].split('\n}', 1)[0]
must(triangle, ['(positions[1] - positions[0]).cross(positions[2] - positions[0])', 'geometric_normal.dot(normal) < 0.0', '[base, base + 2, base + 1]'])
household = src('adventuresim-building-generator/src/furniture/apparatus/household.rs')
must(household, ['pub(super) fn candle_stand', '(0.035, 0.06)', 'builder.turned('])
furniture = src('adventuresim-building-generator/src/furniture.rs')
must(furniture, ['FurnitureKind::CandleStand', 'apparatus::assemble(&mut builder, self)?', 'let mut recipe = builder'])
builder = src('adventuresim-building-generator/src/furniture/builder.rs')
must(builder, ['ResolvedItemId(self.colliders.len() as u64 + 1)', 'self.colliders.push(cuboid)'])
pose = src('adventuresim-building-generator/src/furniture/placement.rs')
must(pose, ['source: local.source', 'Quat::from_rotation_y(self.yaw.radians())'])
physical = src('adventuresim-tactical-core/src/scene_input/furniture/collision.rs')
must(physical, ['Collider::compound(', '.colliders', 'cuboid.centre.metres()', 'cuboid.size.metres().x'])
server = src('adventuresim-tactical-server/src/furniture.rs')
must(server, ['furniture_collider(furniture.key)?', 'for instance in layout.instances', 'commands.spawn(('])
rooms = src('adventuresim-tactical-client/src/tactical_scene_viewer/interior_furniture_capture/rooms.rs')
must(rooms, ['.enumerate()', 'owner: Owner::Furniture(index)', 'solid: pose.cuboid(solid)?'])
if rooms.count('owner: Owner::Furniture(index)') != 2:
    raise ValueError('subject and blocker ownership changed')
sightlines = src('adventuresim-tactical-client/src/tactical_scene_viewer/interior_furniture_capture/rooms/sightlines.rs')
must(sightlines, ['Furniture(usize)', 'blocker.owner == owner'])
presentation = src('adventuresim-tactical-client/src/presentation/furniture.rs')
must(presentation, ['cache.0.entry(instance.key)', '.entity(event.entity)', 'recipe_mesh(batch, Vec3::ZERO)'])
if Fraction(6, 100) ** 2 * Fraction(1, 2) != Fraction(18, 10000):
    raise ValueError('ideal sector illustration changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for historical in doc['historical_sources']:
        index = historical['original_role_index']
        sha = historical['sha256']
        role = roles[index]
        raw = tar.extractfile(sha).read()
        path = historical['source_id'].split(':', 1)[1]
        if (role['source_id'] != historical['source_id'] or role['historical_sha256'] != sha or
            role['historical_assessment'] != historical['assessment'] or archive_hashes[index] != sha or
            hashlib.sha256(raw).hexdigest() != sha or hashlib.sha256(sources[path].encode()).hexdigest() != sha):
            raise ValueError('historical role relationship mismatch')
if len(doc['historical_sources']) != 6 or len(doc['dispositions']) != 2:
    raise ValueError('incorrect disposition scope')
print('verified 13 Git-bound sources, six historical roles and ideal cap arithmetic; no product ran')
