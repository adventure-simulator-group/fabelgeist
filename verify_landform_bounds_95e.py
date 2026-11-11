#!/usr/bin/env python3
"""Verify landform bounds, historical receipt and source provenance only."""
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
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex())
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
name = 'ASSESSMENT-LANDFORM-BOUNDS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1868-1876.json'
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
lengths = {1868:66, 1869:112, 1870:71, 1871:132, 1872:395, 1873:18, 1874:35, 1875:65, 1876:30}
ranges = {1868:[], 1869:[[1,59]], 1870:[[1,98]], 1871:[], 1872:[], 1873:[], 1874:[], 1875:[], 1876:[]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed'
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert (before != body if i == 1870 else before == body)
        assert c['byte_identical'] == (i != 1870) and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == ([] if i == 1870 else [[1, n]])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == (98 if i == 1870 else n)
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
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    old_runtime = archive.extractfile(h['historical_source_roles'][2]['sha256']).read()
assert b'runtime_inverse_kinematics_config' not in old_runtime
assert b'runtime_inverse_kinematics_config' in current[1870]
diff = a['historical_current_diff']; raw_diff = (root / diff['path']).read_bytes()
assert sha(raw_diff) == diff['sha256'] and len(raw_diff) == diff['bytes']
assert b'runtime_inverse_kinematics_config' in raw_diff
assert b'pub enum MuscleRelease' in current[1869] and b'MuscleRelease::Never => None' in current[1869]
nonfault = current[1872]
assert b'const MAX_GRID_SIDE: usize = 195' in nonfault
assert nonfault.index(b'for z in 0..side {') < nonfault.index(b'if side > MAX_GRID_SIDE')
assert b'height_at_extended_edge(terrain, point)?' in nonfault
fault = sources['crates/adventuresim-tactical-core/src/volumetric_terrain.rs']
assert b'const SCARP_EROSION_STEPS: usize = 224' in fault
assert fault.index(b'let surface = simulate_fault_scarp(') < fault.index(b'if side > 195 || vertical > 96')
assert b'for _ in 0..SCARP_EROSION_STEPS' in fault
assert b'horizontal_transfers\n            .par_iter_mut()' in fault and b'offsets\n            .par_iter_mut()' in fault
recipe = sources['crates/adventuresim-tactical-core/src/volumetric_terrain/recipe.rs']
assert b'Self::Detail => 50' in recipe and b'(400..=5_000).contains(&self.half_length_cm)' in recipe
assert b'(300..=2_000).contains(&self.half_width_cm)' in recipe and b'u32::from(self.collar_cm) * 2' in recipe
assert b'recipe.validate(terrain)?' in fault and b'recipe.kind != TerrainLandformKind::FaultScarp' in fault
# Evidence arithmetic only; no terrain, sampler, simulation or product function is invoked.
assert (2 * 52.5 / 0.5) + 1 == 211
assert 211 * 211 == 44521
assert (2 * 52 / 0.5) + 1 == 209
assert 209 * 209 * 224 * 2 == 19569088
seed = sources['crates/fabelgeist-determinism/src/seed.rs']
assert b'field.len() as u64' in seed and b'value.to_le_bytes()' in seed
server = sources['crates/adventuresim-tactical-server/src/doors.rs']
assert b'exemptions.grant(door_entity)' in server and b'exemptions.revoke(door_entity)' in server
assert b'.excluded_entities.insert(door_entity)' in server and b'.excluded_entities.remove(&door_entity)' in server
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(614,[1868,1869,1870,1871]), (615,[1872]), (616,[1873,1874,1875,1876])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]]
        assert row['same_lens_reuse'] == []
adler = json.loads(records['adler/READ-L11-284.json'])
assert [x['original_index'] for x in adler['sources']] == list(range(1871,1877))
for row in adler['sources'] + adler.get('connected_reinspection',[]):
    registration = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert registration in records and row['registration_sha256'] == lookup[registration]['sha256']
prior = load('ASSESSMENT-COMBAT-CONFIG-ERROR-95E.json')
rec = next(x for x in prior['recovered_historical_records'] if x['original_path'] == 'adler/READ-L11-283.json')
assert sha(contents(rec['preserved_bytes'])) == rec['published_sha256'] and rec['sha256'] == lookup[rec['original_path']]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in (1550,1551,1552,1553,1558):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings = a['conditional_findings']
assert [x['id'] for x in findings] == ['REVIEW-EROSION-SIDE-GUARD-LATE-001','REVIEW-FAULT-SIDE-GUARD-LATE-001']
assert all(x['accepted_historical_unique_count_change'] == 0 and x['repair'] == 'not started' and x['validation'] == 'not performed' for x in findings)
assert '44,521' in findings[0]['trace'] and '19,569,088' in findings[1]['trace']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert all(sum(x['id'] == f['id'] and x['record'] == name for x in ledger) == 1 for f in findings)
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified nine historical roles, changed runtime, attributed receipts and bounded prework arithmetic; no product execution.')
