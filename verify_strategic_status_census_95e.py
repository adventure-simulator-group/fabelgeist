#!/usr/bin/env python3
"""Verify source-only census and asset evidence; no product is executed."""
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
name = 'ASSESSMENT-STRATEGIC-STATUS-CENSUS-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1687-1691.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('STRATEGIC-STATUS-CENSUS-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1687: 65, 1688: 133, 1689: 93, 1690: 94, 1691: 130}
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
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1687-1691.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] == (i != 1688)
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        if i != 1688:
            assert body == old
        else:
            assert body != old and c['lines'] == 65
            patch = contents(c['preserved_diff'])
            assert b'-pub(crate) struct CanvasRect {' in patch and b'+pub(crate) use crate::presentation::canvas_view::CanvasRect;' in patch
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i], old_bodies[i] = body, old
    future = a['earlier_historical_source_identities'][0]
    assert future['original_index'] == 1692 and future['new_personal_historical_full_read'] is False and future['additional_original_frontier_credit'] == 0
    old = archive.extractfile(future['sha256']).read()
    assert sha(old) == w['roles'][1692]['historical_sha256']
    chain(future['source_id'].split(':', 1)[1], proof['paths'][future['source_id'].split(':', 1)[1]], old, proof['root_tree'], nodes)

sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
old = old_bodies[1688].decode()
canvas = sources['crates/adventuresim-tactical-client/src/presentation/canvas_view.rs'].decode()
start = '#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]'
left = old[old.index(start):old.index('#[derive(Clone, Debug, Deserialize)]')].replace('pub(super) fn uncropped', 'pub(crate) fn uncropped')
right = canvas[canvas.index(start):canvas.index('#[cfg(test)]')]
assert left == right and old[old.index('#[cfg(test)]'):] == canvas[canvas.index('#[cfg(test)]'):]

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
path = 'segovb/READ-L12-538.json'
receipt = json.loads(records[path])
check = json.loads(records['segovb/VERIFY-L12-538.json'])
assert [row['original_index'] for row in receipt['rows']] == [1689, 1690, 1691, 1692]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['segovb/STATE-L12-538.json']['sha256']
for row in receipt['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
path = 'adler/READ-L11-244.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-244.json'])
assert [row['original_index'] for row in receipt['sources']] == list(range(1690, 1696))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-244.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']
path = 'segovb/READ-L11-623.json'
receipt = json.loads(records[path])
check = json.loads(records['segovb/VERIFY-L11-623.json'])
assert receipt['source_revision'] == 'e2755a732c84e67296b5e3441a4259a84f461b2f'
assert receipt['new_target_reconciliation_complete'] is False and receipt['accepted_unique_count_change'] == 0
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['segovb/STATE-L11-623.json']['sha256']
assert receipt['proposal'] in records and receipt['proposal_sha256'] == lookup[receipt['proposal']]['sha256']
assert receipt['agent_verification'] in records and receipt['agent_verification_sha256'] == lookup[receipt['agent_verification']]['sha256']
for item in receipt['plans']:
    assert item['path'] in records and item['sha256'] == lookup[item['path']]['sha256']
assert any(item['id'] == 'ADLER-L11-STRATEGIC-ASSET-FAILURE-CAUSE-OWNERSHIP' for item in receipt['finding_connections_and_scope_dispositions'])
assert any('lens11_reconcile' in row['attribution'] for row in receipt['rows'])

native = load(a['native_source']['manifest'])
assert native['archive_sha256'] == '1624eddbf2293441b2158506aba75e523432b898d9da26eb6ed3e20af4c7fd38'
event_item = next(x for x in native['inspected_members'] if x['archive_member'] == a['native_source']['event_member'])
event = contents(event_item['preserved_bytes'])
assert sha(event) == event_item['sha256'] == '22ff08ca5948660f8ea0ea2a1a0e9c977b3c9a44bd8c5c538014a9b8200ed0ca'
assert len(event) == event_item['bytes'] and event_item['personally_inspected_ranges'] == [[1, 90]]
server_item = next(x for x in native['inspected_members'] if x['archive_member'] == 'bevy_asset-0.19.1/src/server/mod.rs')
server = contents(server_item['preserved_bytes'])
assert sha(server) == server_item['sha256'] and [2340, 2435] in server_item['personally_inspected_ranges']
assert b'pub id: UntypedAssetId' in event and b'pub error: AssetLoadError' in event and b'error: value.error.clone()' in event
assert b'pub enum AssetLoadError' in server
status = sources['crates/adventuresim-tactical-client/src/strategic_scene/status.rs']
assert b'failure.0 = Some(format!("Could not load scene asset {}", path.display()));' in status
assert b'&& failure.0.is_none()' in status and b'.or_else(|| failure.0.clone())' in status
assert b'pub(super) enum LoadingFailure' in sources['crates/adventuresim-tactical-client/src/loading_screen/progress.rs']
assert b'LoadingFailure::Asset(path.display().to_string())' in sources['crates/adventuresim-tactical-client/src/loading_screen.rs']
assert b'parts: Query<(Entity, &Mesh3d, Option<&ViewVisibility>)>' in current[1689]
assert b'indices.len() == 36' in current[1689] and b'City mesh census {name}' in current[1689]
assert b'Mesh3d(meshes.add(mesh))' in sources['crates/adventuresim-tactical-client/src/browser_runtime.rs']
assert 12 * 3 == 36
assert b'.remove::<(NotShadowCaster, InactiveCharacterShadow)>()' in current[1690]
assert b'.unwrap_or(0.0)' in current[1691]
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [91, 1409, 1410, 1411, 1412, 1413, 1414])
asset, scope, cuboid = a['conditional_findings']
assert [x['id'] for x in [asset, scope, cuboid]] == ['ADLER-L11-STRATEGIC-ASSET-FAILURE-CAUSE-OWNERSHIP', 'REVIEW-CITY-MESH-CENSUS-SCOPE-001', 'REVIEW-CITY-MESH-CENSUS-CUBOID-HEURISTIC-001']
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [asset, scope, cuboid])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [asset, scope, cuboid])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, exact prior attribution and bounded status/census evidence; no product execution.')
