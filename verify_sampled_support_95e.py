#!/usr/bin/env python3
"""Verify source provenance and qualified sampled support evidence only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end+1:end+21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-SAMPLED-SUPPORT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1844-1848.json'
a, h, worklist = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1844:454,1845:318,1846:177,1847:185,1848:37}
ranges = {1844:[],1845:[[1,145],[154,279]],1846:[[1,80],[90,170]],1847:[],1848:[]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i == 1845 else 'unassessed')
        assert item['prior_current_observations'] == row.get('current_observations',[])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert b'assert!(!foundation.cut_faces.is_empty())' in current[1844]
assert b'"cut face ray distance {hit}"' in current[1844]
assert b'pub fn height_at(&self, local: Vec2) -> Option<GeometryResult<SupportElevation>>' in current[1845]
assert b'.ok_or(GeometryError::InvalidProjection)' in current[1845]
assert b'.validate_encoded()' in current[1846] and b'self.validate_encoded()' in current[1846]
assert b'ScenePlanPolygon::from_ordered_vertices(region.outline().vertices().to_vec()).is_err()' in current[1847]
assert b'Dir3::new(normal).ok()?' in current[1848]
scene = sources['crates/adventuresim-tactical-core/src/scene.rs']
admission = sources['crates/adventuresim-tactical-core/src/scene/admission.rs']
queries = sources['crates/adventuresim-tactical-core/src/scene/queries.rs']
assert b'pub fn from_heightmap(' in scene and b'terrain.validate().ok()?' in scene
assert b'|| scale <= 0.0' in admission and b'if self.heightmap.iter().any(|h| !h.is_finite())' in admission
assert b'let normal = tangent_z.cross(tangent_x);' in queries
assert b'SurfaceHit::from_geometry(height, normal)' in queries
assert b'.ok_or(TerrainSupportError::MissingSupport { point })' in queries
scene_input = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'!(0.25..=2_000.0).contains(&grid.spacing_metres)' in scene_input
assert b'terrain.required_height_at(boundary)?' in sources['crates/adventuresim-tactical-core/src/volumetric_terrain/support.rs']
assert b'GardenSupportError::MissingSoil' in sources['crates/adventuresim-tactical-core/src/scene_input/gardens/support.rs']
native = a['native_source_evidence']
with tarfile.open(root / 'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate', 'r:gz') as archive:
    direction = archive.extractfile('bevy_math-0.19.1/src/direction.rs').read()
    assert b'(length.is_finite() && length > 0.0).then_some(value / length)' in direction
    assert b'InvalidDirectionError::Zero' in direction
with tarfile.open(root / 'native-evidence/glam-0.32.1/glam-0.32.1.crate', 'r:gz') as archive:
    v3 = archive.extractfile('glam-0.32.1/src/f32/vec3.rs').read()
    math = archive.extractfile('glam-0.32.1/src/f32/math.rs').read()
    assert b'y: self.z * rhs.x - rhs.z * self.x' in v3
    assert b'x: self.x.div(rhs)' in v3 and b'math::sqrt(self.dot(self))' in v3
    assert sha(math) == native['glam_math_member']['sha256'] and len(math) == native['glam_math_member']['bytes']
    assert b'libm::sqrtf(f)' in math and b'f32::sqrt(f)' in math
    assert b'pub(crate) fn sqrt(_: f32) -> f32 {\n        unimplemented!()' in math
libm = native['libm_archive']; crate = (root / libm['path']).read_bytes()
assert sha(crate) == libm['sha256'] == 'b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981'
assert len(crate) == libm['bytes']
with tarfile.open(root / libm['path'], 'r:gz') as archive:
    for member in libm['inspected_members']:
        body = archive.extractfile(member['path']).read()
        assert sha(body) == member['sha256'] and len(body) == member['bytes']
        assert all(1 <= lo <= hi <= len(body.splitlines()) for lo, hi in member['ranges'])
    generic = archive.extractfile('libm-0.2.16/src/math/generic/sqrt.rs').read()
    assert b'if ix << 1 == zero {' in generic and b'return FpResult::ok(x);' in generic
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(604,[1844,1845]),(605,[1846,1847,1848])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
adler = json.loads(records['adler/READ-L11-278.json'])
assert [x['original_index'] for x in adler['sources']] == list(range(1844,1849))
prior8 = json.loads(records['evidence/continuation-segovb/audit-resume-20261027/lens08/batch-016-originals1894-1906.json'])
assert prior8['lens'] == 8 and prior8['global_clearance'] is False
revisit = json.loads(records['segovb/READ-REVISIT-L12-633-015.json'])
assert revisit['fresh_personal_body_reads'] == [[1,81],[550,611]]
assert revisit['function_contract']['unread_file_ranges'] == [[82,549],[612,1511]]
native_read = json.loads(records['segovb/READ-NATIVE-L12-634-003.json'])
assert native_read['fresh_personal_body_reads'] == [[240,280],[540,575],[1410,1430]]
assert 'unread' in native_read['function_contract']['partial_unassessed_neighbor']
q = load('RECONCILIATION-059.json')['entries']
for i in [5,1506,*range(1531,1537)]:
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-TERRAIN-CUTFACE-95E.json' in q[1506]['current_reconciliation']
finding = a['conditional_findings'][0]
assert finding['id'] == 'CONT08-SAMPLED-SUPPORT-FAILURE-CLASSIFICATION'
assert finding['accepted_historical_unique_count_change'] == 0 and 'not constructed' in finding['trace']
assert 'no actual feature resolution' in finding['safeguards_and_limits']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, qualified native receipts and sampled-support source trace; no product execution.')
