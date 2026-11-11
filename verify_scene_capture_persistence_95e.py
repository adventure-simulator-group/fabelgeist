#!/usr/bin/env python3
"""Verify capture source and original evidence links; no product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
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
name = 'ASSESSMENT-SCENE-CAPTURE-PERSISTENCE-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1700-1705.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('CAPTURE-PERSISTENCE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1700: 224, 1701: 301, 1702: 76, 1703: 97, 1704: 373, 1705: 54}
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
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1700-1705.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    item = a['current_manifest_source']
    old = archive.extractfile(item['historical_sha256']).read()
    body = contents(item['preserved_bytes'])
    assert item['original_index'] == 1719 and item['historical_personal_full_read'] is False and item['additional_original_frontier_credit'] == 0
    assert sha(old) == w['roles'][1719]['historical_sha256'] and oid('blob', old) == item['historical_git_blob']
    assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'] == tree[item['path']]
    assert body != old and len(body.splitlines()) == len(old.splitlines()) == 941
    chain(item['path'], proof['paths'][item['path']], old, proof['root_tree'], nodes)
    assert item['ranges'] == [[1,60],[160,205],[250,430],[487,518],[675,724]]
    assert b'finalize_after_screenshot' in body and b'validation.settled_readback_pairs_identical' in body
    assert b'validation.passed = validation_passes(validation)' in body

diff = (root / a['current_manifest_diff']).read_bytes()
assert diff.count(b'-                    distance: crate::camera::authored_metres(50.0),') == 2
assert diff.count(b'+                    distance: crate::camera::config::authored_metres(50.0),') == 2
assert b'fn deterministic_readback_hash' in current[1704] and b'wrapping_mul' in current[1704]
assert b'commands.write_message(AppExit::error())' in current[1705]
viewer = contents(load('ASSESSMENT-L01-HISTORICAL-1699-COMPLETE.json')['current_comparison']['preserved_bytes'])
assert viewer.index(b'save_to_disk(&path)(captured);') < viewer.index(b'state.view += 1;', viewer.index(b'save_to_disk(&path)(captured);'))
assert b'finish_capture(&state.output, &manifest, valid, &mut exit)' in viewer
assert b'if valid {\n        println!("TACTICAL_SCENE_CAPTURE_VALID={}"' in viewer
native = load(a['native_source'])
raw = (root / 'native-evidence/bevy_render-0.19.1' / native['archive']).read_bytes()
assert sha(raw) == native['sha256']
with tarfile.open(root / 'native-evidence/bevy_render-0.19.1' / native['archive'], 'r:gz') as package:
    screenshot = package.extractfile('bevy_render-0.19.1/src/view/window/screenshot.rs').read()
assert sha(screenshot) == 'f49e4ca4f0929fc51a5c55fc002294c6009ae7a4c1c07dc6ef6c8ff737df0c5b'
assert b'pub fn save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)' in screenshot
assert b'Err(e) => error!("Cannot save screenshot, IO error: {e}")' in screenshot
assert b'Err(e) => error!("Cannot save screenshot, screen format cannot be understood: {e}")' in screenshot

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
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
for n, indices in [(544,[1700,1701]), (545,[1702,1703,1704]), (546,[1705,1706,1707]), (552,[1719])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('partial_sequence', []):
        assert x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
    for x in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
        assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
native_read = json.loads(records['segovb/READ-NATIVE-L12-552-001.json'])
assert native_read['fresh_personal_body_reads'] == [[1,702]] and native_read['current_product_source_reconciled'] is False
assert native_read['accepted_unique_count_change'] == 0
assert [x['original_index'] for x in native_read['qualified_supporting_same_lens_reuse']] == [1699,1719]
for x in native_read['qualified_supporting_same_lens_reuse']:
    assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
    assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
check = json.loads(records['segovb/VERIFY-NATIVE-L12-552-001.json'])
assert check['receipt_sha256'] == lookup['segovb/READ-NATIVE-L12-552-001.json']['sha256']
assert check['registration_sha256'] == lookup['segovb/REGISTRATION-NATIVE-L12-552-001.json']['sha256']
assert check['selector_sha256'] == lookup['segovb/SELECTOR-L12-EXTENSIONS-552.json']['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert q[257]['current_reconciliation'].startswith('partial ') and name in q[257]['current_reconciliation']
finding = a['existing_finding_connections'][0]
assert finding['id'] == 'SEGOVB-L12-SCENE-CAPTURE-PERSISTENCE-OUTCOME'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified six historical roles, native save outcome trace and original reviewer evidence; no product execution.')
