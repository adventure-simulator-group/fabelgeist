#!/usr/bin/env python3
"""Verify source identities and recovery qualifications; no product execution."""
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
name = 'ASSESSMENT-CITY-GROUND-WIRE-ADMISSION-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1662-1666.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('CITY-GROUND-WIRE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1662: 153, 1663: 331, 1664: 175, 1665: 295, 1666: 156}
ranges = {1662: [[1, 153]], 1663: [[1, 170], [171, 331]], 1664: [[1, 175]], 1665: [[1, 160], [161, 295]], 1666: [[1, 156]]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i == 1665 else 'unassessed')
        if i == 1665:
            assert item['prior_current_observations'] == [{'assessment': 'ASSESSMENT-L1-TRIANGLE-COUNTS.json', 'ranges': [[145, 170]]}]
        assert item['new_personal_read_ranges'] == ranges[i] and len(old.splitlines()) == lengths[i]
        assert sha(old) == row['historical_sha256'] and row['status'] == 'full-historical-body-trace-open'
        assert row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1662-1666.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == lengths[i]
        assert c['current_personal_ranges'] == ranges[i] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == list(range(1667, 1672))
    for item in a['earlier_historical_source_identities']:
        row = w['roles'][item['original_index']]
        body = archive.extractfile(item['sha256']).read()
        path = item['source_id'].split(':', 1)[1]
        assert item['new_personal_historical_full_read'] is False and item['additional_original_frontier_credit'] == 0
        assert sha(body) == row['historical_sha256']
        chain(path, proof['paths'][path], body, proof['root_tree'], nodes)
    oldlock = archive.extractfile(w['roles'][497]['historical_sha256']).read()
    lockref = a['historical_lock_excerpt']
    assert lockref['personally_inspected_ranges'] == [[2480, 2483]]
    assert w['roles'][497]['status'] == 'lock-metadata-partial'
    chain('Cargo.lock', proof['paths']['Cargo.lock'], oldlock, proof['root_tree'], nodes)
    assert sha(oldlock) == lockref['sha256'] and oid('blob', oldlock) == lockref['git_blob']

sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
checksum = 'e2adbf8e5a5817c345986756eb0377e18a510e1d7685ef62d6c7bafcd3bab85e'
native = load('native-evidence/bevy_mesh-0.19.1/MANIFEST.json')
assert native['sha256'] == checksum and native['personally_inspected_ranges']['bevy_mesh-0.19.1/src/mesh.rs'] == [[350, 409], [895, 960], [1207, 1267]]
assert checksum.encode() in b'\n'.join(oldlock.splitlines()[2479:2483])
assert checksum.encode() in b'\n'.join(sources['Cargo.lock'].splitlines()[2311:2315])
raw = (root / 'native-evidence/bevy_mesh-0.19.1' / native['archive']).read_bytes()
assert sha(raw) == checksum and len(raw) == native['bytes']
with tarfile.open(fileobj=__import__('io').BytesIO(raw), mode='r:gz') as archive:
    for member in native['inspected_members']:
        body = archive.extractfile(member['path']).read()
        assert sha(body) == member['sha256'] and len(body) == member['bytes']
    mesh = archive.extractfile('bevy_mesh-0.19.1/src/mesh.rs').read()

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
for n, indices in [(527, [1662, 1663, 1664]), (528, [1665, 1666, 1667])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for row in receipt.get('partial_sequence', []):
        assert row['path'] in records and row['sha256'] == lookup[row['path']]['sha256']
path = 'adler/READ-L11-240.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-240.json'])
assert [row['original_index'] for row in receipt['sources']] == list(range(1666, 1672))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-240.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']

binary = sources['crates/adventuresim-tactical-core/src/geometry_transport/binary.rs']
worker = sources['crates/adventuresim-tactical-client/src/presentation/generation/worker_products.rs']
city = sources['crates/adventuresim-tactical-client/src/presentation/generation/city.rs']
reflected = sources['crates/adventuresim-tactical-client/src/presentation/vista/geographic_ground.rs']
js = sources['crates/strategic-web/static/strategic-generation.js']
assert b'!bytes.len().is_multiple_of(size_of::<T>())' in binary
assert b'#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]' in current[1663]
assert all(x in current[1663] for x in (b'vertices.positions.push(', b'vertices.normals.push(', b'vertices.uvs.push(', b'vertices.footprints.push(', b'vertices.activities.push('))
assert b'PrimitiveTopology::TriangleList' in current[1663] and b'geographic_ground::reflected(mesh)?' in current[1665]
assert b'ciborium::from_reader(bytes)' in worker and b'city.matches(&input, &graphics)' in worker
assert b'self.document == *document && self.graphics == graphics' in city
assert b'mesh.count_vertices()' in reflected and b'mesh.invert_winding()?' in reflected
assert b'core::cmp::min(previous_vertex_count, attribute_len)' in mesh and b'MeshWindingInvertError::AbruptIndicesEnd' in mesh
assert b'runtime.wasm_receive_job(preparation, job, bytes)' in js and b'invalidCachedProductCodes.has(error?.name)' in js
assert [3 * 12, 2 * 12, 3 * 8, 3 * 16, 3 * 8] == [36, 24, 24, 48, 24]
assert min(3, 2, 3, 3, 3) == 2 and 2 % 3 != 0
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1384, 1389))
finding, = a['conditional_findings']
assert finding['id'] == 'Q-L12-ba25fccbf4ff81c29772'
assert load('baseline-059/queue.json')['entries'][1385]['contract_id'] == finding['id']
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified city-ground source, historical attribution, pinned native evidence and symbolic counts; no product execution.')
