#!/usr/bin/env python3
"""Verify preserved audit evidence; this does not execute the product."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
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
name = 'ASSESSMENT-STRATEGIC-ADAPTER-DIAGNOSTICS-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1682-1686.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('STRATEGIC-ADAPTER-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1682: 159, 1683: 150, 1684: 95, 1685: 40, 1686: 298}
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
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1682-1686.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] == (i != 1686)
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        if i != 1686:
            assert body == old
        else:
            assert body != old and c['lines'] == 296
            assert old.replace(b'#[cfg(target_family = "wasm")]\npub(crate) use views::StrategicCamera;\n', b'', 1) == body
            patch = contents(c['preserved_diff'])
            assert b'-#[cfg(target_family = "wasm")]' in patch and b'-pub(crate) use views::StrategicCamera;' in patch
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
prior = load('ASSESSMENT-STRATEGIC-VIEW-RETENTION-95E.json')
assert [x['original_index'] for x in prior['earlier_historical_source_identities']] == [1682, 1683]
assert all(x['new_personal_historical_full_read'] is False and x['additional_original_frontier_credit'] == 0 for x in prior['earlier_historical_source_identities'])

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
path = 'segovb/READ-L12-537.json'
receipt = json.loads(records[path])
check = json.loads(records['segovb/VERIFY-L12-537.json'])
assert [row['original_index'] for row in receipt['rows']] == [1686, 1687, 1688]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['segovb/STATE-L12-537.json']['sha256']
for row in receipt['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
assert b'nextpersonlayerusize' in records[path]
path = 'adler/READ-L11-243.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-243.json'])
assert [row['original_index'] for row in receipt['sources']] == list(range(1684, 1690))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-243.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']

native = load(a['native_source']['manifest'])
member = a['native_source']['member']
assert native['sha256'] == '96de20d0babfb51e3907eb4eb56f1e36096a6f8c77acd997f5de611df4ee067d'
archive_path = root / 'native-evidence/bevy_camera-0.19.1' / native['archive']
assert sha(archive_path.read_bytes()) == native['sha256']
with tarfile.open(archive_path, 'r:gz') as package:
    raw = package.extractfile(member).read()
entry = next(x for x in native['inspected_members'] if x['path'] == member)
assert sha(raw) == entry['sha256'] and len(raw) == entry['bytes']
assert native['personally_inspected_ranges'][member] == [[1, 205]]
assert b'SmallVec<[u64; INLINE_BLOCKS]>' in raw and b'let buffer_index = layer / 64;' in raw
assert b'self.extend_buffer(buffer_index + 1);' in raw and b'self.0.resize(new_size, 0u64);' in raw

views = sources['crates/adventuresim-tactical-client/src/strategic_scene/views.rs']
staging = sources['crates/adventuresim-tactical-client/src/strategic_scene/staging.rs']
protocol = sources['crates/adventuresim-tactical-client/src/strategic_scene/protocol.rs']
ownership = sources['crates/adventuresim-tactical-client/src/presentation/ownership.rs']
lighting = sources['crates/adventuresim-tactical-client/src/presentation/interior_lighting/mod.rs']
world = sources['crates/adventuresim-tactical-client/src/strategic_scene/world.rs']
status = sources['crates/adventuresim-tactical-client/src/strategic_scene/status.rs']
js = sources['crates/strategic-web/static/strategic-scene.js']
assert b'const PORTRAIT_DISTANCE_METRES: f32 = 0.30;' in views
assert b'pub(super) const CONVERSATION_DISTANCE_METRES: f32 = 1.1;' in staging
assert b'*transform.forward() * super::staging::CONVERSATION_DISTANCE_METRES' in current[1685]
assert Fraction(3, 10) - Fraction(11, 10) == Fraction(-4, 5)
assert b'camera.is_active' in current[1685] and b'view_lighting: lighting.views()' in status
assert b'window.strategicRendererMetrics = metrics;' in js and b'metrics.state = state;' in js
assert b'crate::presentation::canvas_view::StrategicCamera' in sources['crates/adventuresim-tactical-client/src/presentation/regional_map/mod.rs']
assert b'cell.floor().as_uvec3()' in lighting and b'Vec3::ZERO' in lighting
assert b'scene.people.retain(' in current[1686] and b'scene.next_person_layer += 1;' in current[1686]
assert b'RenderLayers::none().with(layer)' in current[1686] and b'world.insert_resource(RetainedScene::default())' in current[1686]
assert b'REGIONAL_MAP_LAYER: usize = 3;' in ownership and b'FIRST_PERSON_LAYER: usize = REGIONAL_MAP_LAYER + 1;' in protocol
assert (4 + 65536 - 1) // 64 + 1 == 1025 and 1025 * 8 == 8200
assert b'filter_map(|(index, row)|' in current[1683] and b'parent: *items.get(parent)?' in current[1683]
assert b'#[primary_key]\n    pub inventory_item_id: u64' in sources['crates/adventuresim-stdb-module/src/character.rs']
assert b'compile_operable_doors(&building.plan)?' in current[1684] and b'compile_operable_windows(&building.plan)?' in current[1684]
assert b'scene.error = Some(Arc::new(PreparationError::Presentation(error)));' in world and b'&& scene.error.is_none()' in status
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1404, 1409))
light, layers = a['conditional_findings']
assert (light['id'], layers['id']) == ('REVIEW-STRATEGIC-PORTRAIT-DIAGNOSTIC-SUBJECT-001', 'REVIEW-STRATEGIC-PERSON-LAYER-CHURN-001')
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [light, layers])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [light, layers])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, qualified receipts and source-only diagnostic/layer arithmetic; no product execution.')
