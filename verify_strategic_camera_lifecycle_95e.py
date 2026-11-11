#!/usr/bin/env python3
"""Verify historical and current camera evidence without product execution."""
import gzip
import hashlib
import json
import math
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-STRATEGIC-CAMERA-LIFECYCLE-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1692-1696.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('STRATEGIC-CAMERA-LIFECYCLE-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1692: 226, 1693: 55, 1694: 314, 1695: 361, 1696: 398}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current, old_bodies = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1, lengths[i]]]
        assert len(old.splitlines()) == lengths[i] and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1692-1696.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] == (i not in (1692, 1696))
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        if i not in (1692, 1696):
            assert body == old
        else:
            assert body != old
            patch = contents(c['preserved_diff'])
            if i == 1692:
                assert b'+        if is_required_scene_asset(path)' in patch and b'+    let waiting = waiting_pipelines();' in patch
            else:
                assert b'+pub(crate) use crate::presentation::canvas_view::StrategicCamera;' in patch and b'-            front: Vec3::ZERO,' in patch
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i], old_bodies[i] = body, old
    future = a['earlier_historical_source_identities'][0]
    assert future['original_index'] == 1697 and future['new_personal_historical_full_read'] is False and future['additional_original_frontier_credit'] == 0
    old = archive.extractfile(future['sha256']).read()
    assert sha(old) == w['roles'][1697]['historical_sha256']
    chain(future['source_id'].split(':', 1)[1], proof['paths'][future['source_id'].split(':', 1)[1]], old, proof['root_tree'], nodes)
previous = load('ASSESSMENT-STRATEGIC-STATUS-CENSUS-95E.json')['earlier_historical_source_identities'][0]
assert previous['original_index'] == 1692 and previous['new_personal_historical_full_read'] is False and previous['additional_original_frontier_credit'] == 0
assert b'pub front: Position<Scene>' in old_bodies[1694] and b'front: Vec3::ZERO,' in old_bodies[1696]
assert b'front: adventuresim_building_generator::spatial_geometry::Position::from_metres(' in current[1696]

sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(539, [1693, 1694]), (540, [1695]), (541, [1696, 1697])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
assert b'front:Vec3::ZERO' in records['segovb/READ-L12-541.json']
path = 'adler/READ-L11-245.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-245.json'])
assert [row['original_index'] for row in receipt['sources']] == [1696, 1697, 1698]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-245.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']

atm = sources['crates/adventuresim-tactical-client/src/presentation/atmosphere.rs']
vis = sources['crates/adventuresim-tactical-client/src/strategic_scene/visibility.rs']
settings = sources['crates/adventuresim-tactical-client/src/presentation/environment.rs']
cache = sources['crates/adventuresim-tactical-client/src/strategic_scene/cached_views.rs']
assert b'.remove::<(EnvironmentMapLight, CachedAtmosphereProbeAssets)>()' in atm
assert b'.remove::<AtmosphereSettings>()' in atm and b'if !enabled {' in atm
assert b'environment.as_ref().is_some_and(|value| value.is_changed())' in current[1696]
assert b'if let Some(environment) = &environment {' in current[1696] and b'target.insert((**environment).clone());' in current[1696]
assert b'target.remove::<(bevy::pbr::DistanceFog, bevy::pbr::AtmosphereSettings)>()' in current[1696]
assert b'weather changes within an installed city' in vis and b'never close this gate' in vis
assert b'if camera.is_active && ready.0' in cache
assert b'vertical_fov_degrees: 80.0' in settings
street = current[1694]
for fragment in [b'const MINIMUM_BAY_METRES: f32 = 18.0;', b'const SKY_MARGIN_METRES: f32 = 8.0;', b'const EYE_HEIGHT_METRES: f32 = 1.7;', b'const FRAME_CENTRE_HEIGHT_FRACTION: f32 = 0.45;', b'2.0 * (self.height * 0.5 / self.distance()).atan()', b'street.front.metres() + Vec3::Y * 0.01']:
    assert fragment in street
width, height, eye = 18.0, 30.0, 1.7
distance = (width / 2) / math.tan(math.radians(40))
centre = math.atan((0.45 * height - eye) / distance)
half_fov = math.atan((height / 2) / distance)
ground = math.atan((0.01 - eye) / distance)
assert ground < centre - half_fov and 2.2 < math.degrees(centre - half_fov - ground) < 2.3
assert b'view.people.len()' in current[1693] and b'counts.required_posed += 1;' in current[1693]
assert b'counts.posed == counts.loaded' in current[1692]
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1414, 1419))
env, framing = a['conditional_findings']
assert [x['id'] for x in [env, framing]] == ['REVIEW-STRATEGIC-CAMERA-OPTIONAL-ENVIRONMENT-REMOVAL-001', 'REVIEW-STRATEGIC-FRONTAGE-VERTICAL-FRAMING-001']
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [env, framing])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [env, framing])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, earlier attribution and source-only camera arithmetic; no product execution.')
