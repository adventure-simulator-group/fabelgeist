#!/usr/bin/env python3
"""Verify bounded replacement evidence and the historical viewer prefix."""
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
name = 'ASSESSMENT-STRATEGIC-REPLACEMENT-ERROR-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1697-1698.json')
partial = load('ASSESSMENT-L01-HISTORICAL-1699-PARTIAL-001.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('STRATEGIC-REPLACEMENT-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1697: 214, 1698: 177}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1, lengths[i]]]
        assert len(old.splitlines()) == lengths[i] and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1697-1698.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    row = w['roles'][1699]
    old = archive.extractfile(partial['historical_sha256']).read()
    c = partial['current_comparison']
    body = contents(c['preserved_bytes'])
    assert partial['original_index'] == 1699 and partial['lines'] == 4477 and partial['next_unread_line'] == 1601
    assert partial['new_personal_read_ranges'] == [[1, 400], [401, 800], [801, 1200], [1201, 1600]]
    assert sha(old) == row['historical_sha256'] == partial['historical_sha256'] and len(old.splitlines()) == 4477
    assert c['byte_identical'] and body == old and sha(body) == c['sha256'] and len(body) == c['bytes']
    assert oid('blob', body) == c['git_blob'] == tree[c['path']]
    chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
    assert any(x['assessment'] == 'ASSESSMENT-L01-HISTORICAL-1699-PARTIAL-001.json' and x['ranges'] == partial['new_personal_read_ranges'] for x in row['historical_partial_observations'])
    assert 'no additional current semantic' in c['qualification']
previous = load('ASSESSMENT-STRATEGIC-CAMERA-LIFECYCLE-95E.json')['earlier_historical_source_identities'][0]
assert previous['original_index'] == 1697 and previous['new_personal_historical_full_read'] is False and previous['additional_original_frontier_credit'] == 0

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
for n, indices in [(542, [1698]), (543, [1699])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for item in receipt.get('partial_sequence', []):
        assert item['path'] in records and item['sha256'] == lookup[item['path']]['sha256']
    for item in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert item['registration'] in records and item['registration_sha256'] == lookup[item['registration']]['sha256']
        assert item['receipt'] in records and item['receipt_sha256'] == lookup[item['receipt']]['sha256']
assert len(json.loads(records['segovb/READ-L12-543.json'])['partial_sequence']) == 8

world = current[1698]
assert b'if scene.location == view.location && scene.error.is_some()' in world
assert world.index(b'let generated = match document.generated_scene()') < world.index(b'commands.entity(root).despawn()')
assert b'Err(error) => {\n                scene.error = Some(error);\n                return;' in world
assert b'mut generated: crate::presentation::PreparedStrategicScene' in world
status = sources['crates/adventuresim-tactical-client/src/strategic_scene/status.rs']
assert b'&& scene.error.is_none()' in status and b'.error\n        .as_ref()\n        .map(ToString::to_string)' in status
owner = sources['crates/adventuresim-tactical-client/src/presentation/generation/ownership.rs']
assert b'.ok_or(PreparationError::StalePreparation)?' in owner and b'PreparationError::PreparationInputMismatch' in owner
request = sources['crates/strategic-web/static/strategic-scene-request.js']
assert b'preparation = await owned' in request and b'install({ location, input, preparation })' in request
assert b'if (location === residentLocation)' in request
viewer = contents(partial['current_comparison']['preserved_bytes'])
assert b'fn prepare_fresh_output(output: &Path)' in viewer and b'capture output already exists; choose a fresh directory' in viewer
assert b'fs::create_dir_all(output)' in viewer
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [1419, 1420, 1421])
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-STRATEGIC-DESTINATION-ERROR-IDENTITY-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified two historical roles, a qualified viewer prefix and destination error source trace; no product execution.')
