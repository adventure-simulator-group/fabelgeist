#!/usr/bin/env python3
"""Verify tree stream, diagnostic, recovered reviewer and native source evidence."""
import difflib
import gzip
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda value: hashlib.sha256(value).hexdigest()

def oid(kind, value):
    return hashlib.sha1(kind.encode() + b' ' + str(len(value)).encode() + b'\0' + value).hexdigest()

def entries(value):
    out, pos = {}, 0
    while pos < len(value):
        end = value.index(b'\0', pos)
        mode, name = value[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), value[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(value)
    return out

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
a = load('ASSESSMENT-TREE-STREAM-DIAGNOSTICS-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1606-1607.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TREE-STREAM-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, hash_ = header.split()
    tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']; raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1606,1607]
assert [x['lines'] for x in h['historical_source_roles']] == [629,1392]
hr = {1606:[[1,210],[211,420],[421,629]],1607:[[1,250],[251,500],[501,750],[751,1000],[1001,1200],[1201,1392]]}
cr = {1606:[[1,210],[211,345]],1607:[[1,150],[205,385],[399,426],[474,820]]}
current, historic = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1606-1607.json'
        assert x['new_personal_read_ranges'] == hr[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is False and original != body and c['current_personal_ranges'] == cr[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
        d = c['exact_diff']
        assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        current[i], historic[i] = body, original
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1608,1699]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]; body = archive.extractfile(x['sha256']).read(); path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] and x['new_personal_historical_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
native = a['native_source_evidence']; m = load(native['manifest']); archive_path = root / 'native-evidence/bevy_camera-0.19.1' / m['archive']
archive = archive_path.read_bytes(); assert len(archive) == m['bytes'] and sha(archive) == m['sha256']
lock = tomllib.loads(sources['Cargo.lock'].decode())
packages = [p for p in lock['package'] if p['name'] == native['package'] and p['version'] == native['version']]
assert len(packages) == 1 and packages[0]['checksum'] == m['sha256']
with tarfile.open(archive_path, 'r:gz') as tar:
    native_bodies = {}
    for member in (native['member'],native['camera_member']):
        item = next(x for x in m['inspected_members'] if x['path'] == member); body = tar.extractfile(member).read()
        assert len(body) == item['bytes'] and sha(body) == item['sha256']
        assert all(1 <= lo <= hi <= len(body.splitlines()) for lo,hi in m['personally_inspected_ranges'][member])
        native_bodies[member] = body
range_body, camera_body = native_bodies[native['member']], native_bodies[native['camera_member']]
assert b'camera_distance >= self.start_margin.start && camera_distance < self.end_margin.end' in range_body
assert b'if this field is set to false, Bevy will use the origin' in range_body
assert b'pub fn physical_viewport_size(&self) -> Option<UVec2>' in camera_body and b'.or_else(|| self.physical_target_size())' in camera_body

base = 'crates/adventuresim-tactical-client/src/'
presenter, previous = current[1607], historic[1607]
leaf = sources[base+'presentation/obstacles/tree/geometry/leaves.rs']
selection = sources[base+'presentation/obstacles/tree/geometry/leaves/selection.rs']
arrangement = sources[base+'presentation/obstacles/tree/geometry/leaves/arrangement.rs']
params = sources[base+'presentation/obstacles/tree/geometry.rs']
lod = sources[base+'presentation/obstacles/tree/lod.rs']
imp = sources[base+'presentation/obstacles/tree/impostor.rs']
viewer = sources[base+'tactical_scene_viewer.rs']
assert b'TreeLeafTriangleCount(cluster_leaf_count * 2)' in presenter
assert b'procedural_oak_leaf_card_group_mesh(&cached.leaves, cluster.primary_group)' in presenter
assert b'detailed_flat_card_group_leaves(leaves, primary_group)' in leaf
assert b'leaf.leaf_ordinal & 3 != omitted_ordinal_lane' in selection and b'.index(4) as u8' in selection
assert b'let leaves_per_shoot = 16_u64;' in arrangement and b'leaves_per_shoot: 12,' in params
assert b'indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);' in leaf
assert b'Query<(&TreeLeafTriangleCount, &TreeLeafRepresentation)>' in viewer and b'.map(|(triangles, _)| triangles.0)' in viewer
assert b'result.scene_leaf_triangles' in viewer and b'leaf-benchmark.json' in viewer
assert b'const EVERY_TREE_TIER: u8 = 0b11_1111;' in presenter
assert b'cached,\n            EVERY_TREE_TIER,\n            None,' in presenter
assert b'cached,\n            mask,\n            selected_leaf,' in previous
assert b'spawn_streamed_tree_children(&mut commands, entity, cached, new_mask, new_leaf_mask)' in presenter
assert b'.physical_viewport_size()' in lod and b'reference_focal = 720.0' in lod
assert b'(80.0_f32.to_radians() * 0.5).tan() / (projection.fov * 0.5).tan()' in presenter
assert b'u8::from((40.0..220.0).contains(&scaled_distance)) << 5' in presenter
assert b'[6.0..8.0, 12.0..16.0, 24.0..32.0, 50.0..60.0]' in imp
assert b'4 => (transition(3), (190.0 * focal_scale)..(200.0 * focal_scale))' in imp
# Elementary source-derived arithmetic, not execution of the product or a native API.
assert 2160 // 720 == 3 and 24*3 <= 140 < 70*3 and 140 < 50*3
assert 16*2 == 32 and (16 - 16//4)*2 == 24
assert 12*2 == 24 and (12 - 12//4)*2 == 18

manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}; records = {}
for x in a['recovered_historical_records']:
    original = lookup[x['original_path']]
    assert (x['sha256'], x['bytes'], x['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    body = contents(x['preserved_bytes'])
    assert sha(body) == x['published_sha256'] and len(body) == x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body) == x['sha256'] and oid('blob', body) == x['git_blob'])
    records[x['original_path']] = body
for n, indices in [(504,[1606]),(505,[1607]),(506,[1608,1609,1610])]:
    path=f'segovb/READ-L12-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for reused in receipt.get('qualified_supporting_same_lens_reuse',[]):
        assert reused['registration'] in records and reused['registration_sha256'] == lookup[reused['registration']]['sha256']
        assert reused['receipt'] in records and reused['receipt_sha256'] == lookup[reused['receipt']]['sha256']
        assert reused['fresh_personal_body_reads_this_batch'] == []
    previous_part = None
    for j, x in enumerate(receipt.get('partial_sequence',[])):
        assert n == 505 and x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
        part = json.loads(records[x['path']]); assert part['original_index'] == 1607 and part['actual_EOF'] == 1392
        assert part['source_sha256'] == w['roles'][1607]['historical_sha256'] and part['original_credit'] == 0
        assert part['fresh_personal_body_reads'] == [[[1,600]],[[601,1200]],[[1201,1392]]][j]
        assert part['unread_tail'] == [[[601,1392]],[[1201,1392]],[]][j]
        if previous_part is not None:
            assert part['predecessor'] == previous_part
        previous_part = x
for n, indices in [(228,[1603,1604,1605,1606]),(229,[1607]),(230,[1608,1609,1610,1611,1612]),(246,[1699])]:
    path=f'adler/READ-L11-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (83,84,1338,1339))
baseline = load('BASELINE-95E-20261110.json'); ledger = baseline['newly_assessed_ledger']['entries']
assert [x['id'] for x in a['existing_finding_connections']] == ['ADLER-L11-TREE-CARD-TRIANGLE-DIAGNOSTIC-AUTHORITY','ADLER-L11-TREE-STREAMING-PROJECTION-AUTHORITY']
for item in a['existing_finding_connections']:
    assert item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed'
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-TREE-STREAM-DIAGNOSTICS-95E.json' for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified two historical bodies, native camera archive, recorded read scopes and source-only tree diagnostics; no product execution.')
