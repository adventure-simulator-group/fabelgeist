#!/usr/bin/env python3
"""Verify recovered source lineage and symbolic cases; never run the product."""
import difflib
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
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
a = load('ASSESSMENT-PEBBLE-CHUNK-CELESTIAL-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1571-1573.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('PEBBLE-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    head, path = line.split('\t', 1)
    mode, kind, hash_ = head.split()
    tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']
raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1571, 1572, 1573]
assert [x['lines'] for x in h['historical_source_roles']] == [676, 250, 522]
expected_ranges = {1571: [[1, 350], [351, 676]], 1572: [[1, 250]], 1573: [[1, 275], [276, 522]]}
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open'
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1571-1573.json'
        assert x['new_personal_read_ranges'] == expected_ranges[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is False and original != body
        assert c['current_personal_ranges'] == ([[1, 440]] if i == 1571 else [[1, c['lines']]])
        check_chain(c['path'], proof['paths'][c['path']], original)
        diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/' + c['path'], tofile='current/' + c['path'])).encode()
        saved = c['exact_diff']
        assert diff == contents(saved['preserved_bytes']) and sha(diff) == saved['sha256'] and len(diff) == saved['bytes']
        current[i] = body

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/presentation/'
loose, spawn = current[1571], current[1572]
batches = sources[base + 'ground_scatter/loose_stone/batches.rs']
reach = sources[base + 'ground_scatter/instanced_grass.rs']
celestial = sources[base + 'celestial_materials.rs']
scene_input = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
geo = sources['crates/adventuresim-tactical-core/src/scene_input/generation/geographic.rs']
shader = sources['assets/shaders/tactical_pebble_billboard_instanced.wgsl']
assert b'ground.grid_scale() * 0.5' in spawn and b'PEBBLE_BATCH_FOOTPRINT_METRES: f32 = 1.0' in loose
assert b'if woodland { 0.58 } else { 1.0 }' in spawn
assert b'ground_scatter_reach(PEBBLE_BATCH_FOOTPRINT_METRES)' in batches and b'horizontal: footprint' in reach
assert b'const PEBBLE_PATCH_COLUMNS: usize = 7' in loose and b'jitter_x * 0.88' in loose
assert b'(source_width - 1).max(source_depth - 1)' in scene_input and b'(MAX_PLAYABLE_SIDE - 1) / largest_source_side' in scene_input
assert b'const MAX_PLAYABLE_SIDE: usize = 601' in scene_input and b'0.25..=2_000.0' in scene_input
assert b'upsample_playable_grid(&self.playable)' in geo and b'let ground = build_scene_ground(' in geo
assert 600 // 400 == 1 and (Fraction(606, 100) / 7 * 2 - 1) * 2 > 1
assert b'if !celestial.is_changed()' in celestial and b'.is_some_and' in celestial and b'1e-6' in celestial
assert b'pebble_materials.iter_mut()' in celestial and b'lighting: Vec3::new(0.35, 0.86, 0.25)' in spawn
assert b'pebble.lighting.xyz' in shader and b'.after(update_presented_celestial_lighting)' in sources[base + 'mod.rs']

native = load('native-evidence/bevy_camera-0.19.1/MANIFEST.json')
archive_path = root / 'native-evidence/bevy_camera-0.19.1' / native['archive']
assert sha(archive_path.read_bytes()) == native['sha256']
member = next(x for x in native['inspected_members'] if x['path'] == 'bevy_camera-0.19.1/src/visibility/mod.rs')
with tarfile.open(archive_path, 'r:gz') as archive:
    body = archive.extractfile(member['path']).read()
    assert sha(body) == member['sha256'] and len(body) == member['bytes']
assert native['personally_inspected_ranges'][member['path']] == [[748, 855]]

manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for x in a['recovered_historical_records']:
    original = lookup[x['original_path']]
    assert (x['sha256'], x['bytes'], x['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    body = contents(x['preserved_bytes'])
    assert sha(body) == x['published_sha256'] and len(body) == x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body) == x['sha256'] and oid('blob', body) == x['git_blob'])
    records[x['original_path']] = body
for n, indices in [(486, [1571]), (487, [1572, 1573])]:
    receipt = json.loads(records[f'segovb/READ-L12-{n}.json'])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert check['receipt_sha256'] == lookup[f'segovb/READ-L12-{n}.json']['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
adler = json.loads(records['adler/READ-L11-221.json'])
check = json.loads(records['adler/VERIFY-L11-221.json'])
assert [x['original_index'] for x in adler['sources']] == [1571, 1572, 1573]
assert adler['accepted_unique_count_change'] == 0 and adler['current_reconciled'] is False and adler['validated'] is False
assert check['receipt_sha256'] == lookup['adler/READ-L11-221.json']['sha256']
assert check['state_sha256'] == lookup['adler/STATE-L11-221.json']['sha256']
for row in adler['sources']:
    path = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
    assert path in records and row['registration_sha256'] == lookup[path]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1320, 1321))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
p = load('CONTINUATION-PROGRESS-95E.json')
assert len(a['conditional_findings']) == 2
for item in a['conditional_findings']:
    assert item['repair'] == 'not started' and item['validation'] == 'not performed'
    assert item['accepted_historical_unique_count_change'] == 0
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-PEBBLE-CHUNK-CELESTIAL-95E.json' for x in ledger) == 1
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
check = load(baseline['latest_main_recheck'])
assert check['observed_revision'] == revision and check['ref'] == 'refs/heads/main'
print('Verified three historical bodies, earlier attribution, current sources and symbolic questions; no product execution.')
