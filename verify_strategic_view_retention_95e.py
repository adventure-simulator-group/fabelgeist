#!/usr/bin/env python3
"""Verify strategic view evidence, attribution, and symbolic size arithmetic."""
import gzip
import hashlib
import json
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
name = 'ASSESSMENT-STRATEGIC-VIEW-RETENTION-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1677-1681.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('STRATEGIC-VIEW-RETENTION-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1677: 77, 1678: 252, 1679: 529, 1680: 169, 1681: 86}
ranges = {1677: [[1, 77]], 1678: [[1, 252]], 1679: [[1, 175], [176, 350], [351, 529]], 1680: [[1, 169]], 1681: [[1, 86]]}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == ranges[i]
        assert len(old.splitlines()) == lengths[i] and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1677-1681.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == lengths[i]
        assert c['current_personal_ranges'] == ranges[i] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    assert [item['original_index'] for item in a['earlier_historical_source_identities']] == [1682, 1683]
    for item in a['earlier_historical_source_identities']:
        body = archive.extractfile(item['sha256']).read()
        path = item['source_id'].split(':', 1)[1]
        assert item['new_personal_historical_full_read'] is False and item['additional_original_frontier_credit'] == 0
        assert sha(body) == w['roles'][item['original_index']]['historical_sha256']
        chain(path, proof['paths'][path], body, proof['root_tree'], nodes)

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
        assert 'pinned complete059 transport' in item['original_recovery']
    records[item['original_path']] = body
for n, indices in [(533, [1677, 1678]), (534, [1679]), (535, [1680, 1681, 1682]), (536, [1683, 1684, 1685])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
assert b'2/64MiBhistory' in records['segovb/READ-L12-535.json']
path = 'adler/READ-L11-242.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-242.json'])
assert [row['original_index'] for row in receipt['sources']] == list(range(1678, 1684))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-242.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']
prev = load('ASSESSMENT-ROAD-WEATHER-IDENTITY-95E.json')
assert prev['earlier_historical_source_identities'][0]['original_index'] == 1677 and prev['earlier_historical_source_identities'][0]['additional_original_frontier_credit'] == 0

travel = sources['crates/adventuresim-tactical-client/src/strategic_scene/cached_views/travel.rs']
world = sources['crates/adventuresim-tactical-client/src/strategic_scene/world.rs']
status = sources['crates/adventuresim-tactical-client/src/strategic_scene/status.rs']
mod = sources['crates/adventuresim-tactical-client/src/strategic_scene/mod.rs']
views = sources['crates/adventuresim-tactical-client/src/strategic_scene/views.rs']
protocol = sources['crates/adventuresim-tactical-client/src/strategic_scene/protocol.rs']
js = sources['crates/strategic-web/static/strategic-scene.js']
request = sources['crates/strategic-web/static/strategic-scene-request.js']
assert b'Portrait { person: PortraitId, slot: usize }' in current[1679]
assert b'for frame in self.retained.frames.values_mut()' in current[1679] and b'Visibility::Hidden' in current[1679]
assert b'self.retained.frames.remove(key)' in current[1679] and b'self.retained.frames.insert(key.clone(), frame)' in current[1679]
assert b'Image::new_target_texture(' in current[1679] and b'TextureFormat::Rgba8UnormSrgb' in current[1679]
assert b'const RETAINED_IMAGE_BYTES: u64 = 64 * 1024 * 1024;' in travel
assert b'self.previous.iter().map(SavedViews::bytes).sum::<u64>() > RETAINED_IMAGE_BYTES' in travel
assert 65 * 512 * 512 * 4 == 65 * 1024 * 1024 and 65 * 1024 * 1024 > 64 * 1024 * 1024
assert b'if scene.location != view.location || scene.root.is_none()' in world
assert b'retained.next_place = view.places.len();' in world
assert b'scene.next_place == view.places.len()' in status
assert b'let Some(venue) = scene.venues.get(&person.place)' in mod
assert b'and_then(|id| scene.venues.get(id))' in views
assert b'pub places: Vec<Place>' in protocol and b'function placeList(nav)' in js
assert b'if (location === residentLocation)' in request
assert b'world.insert_resource(RetainedScene::default())' in mod
assert b'#[component(immutable)]' in sources['crates/adventuresim-tactical-core/src/scene_input/buildings/window.rs']
assert b'.replicate::<SceneWindow>()' in sources['crates/adventuresim-tactical-netcode/src/replication.rs']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [1399, 1400, 1401, 1402, 1403, 1404, 1420])
images, places = a['conditional_findings']
assert (images['id'], places['id']) == ('REVIEW-STRATEGIC-ACTIVE-VIEW-RETENTION-001', 'REVIEW-STRATEGIC-RETAINED-PLACE-IDENTITY-001')
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [images, places])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [images, places])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified strategic view source, earlier receipts and symbolic image sizes; no product execution.')
