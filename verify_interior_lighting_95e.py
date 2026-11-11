#!/usr/bin/env python3
"""Verify original and physical snapshot lineage and symbolic source claims."""
import difflib
import gzip
import hashlib
import json
import tarfile
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
a = load('ASSESSMENT-INTERIOR-LIGHTING-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1582-1588.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('INTERIOR-HISTORICAL-GIT-PROOFS.json')
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

assert [x['original_index'] for x in h['historical_source_roles']] == list(range(1582,1589))
assert [x['lines'] for x in h['historical_source_roles']] == [357,129,206,82,251,115,331]
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1582-1588.json'
        ranges = {1582:[[1,185],[186,357]],1588:[[1,170],[171,331]]}.get(i,[[1,x['lines']]])
        assert x['new_personal_read_ranges'] == ranges
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i != 1582) and c['current_personal_ranges'] == ranges
        check_chain(c['path'], proof['paths'][c['path']], original)
        if i == 1582:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']
            assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
            assert b'(19_800, 9_900, 4_960, 2_480)' in original and b'(19_800, 9_900, 5_012, 2_506)' in body
        current[i] = body
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1589,1590,1591]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] and x['new_personal_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)
    physical = a['earlier_physical_source_identities']
    assert {x['original_index'] for x in physical} == {2515,2517,2526,2530,2556}
    for x in physical:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        assert x['source_id'] == role['source_id'] and x['source_id'].startswith('physical-p16:')
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['original_assessed_ranges'])
        assert x['new_personal_full_read'] is False and x['additional_original_frontier_credit'] == 0
        assert 'transport only' in x['frozen_snapshot']

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
client = 'crates/adventuresim-tactical-client/src/'
core = 'crates/adventuresim-tactical-core/src/'
gen = 'crates/adventuresim-building-generator/src/'
field, exposure, lighting, shutters = current[1584], current[1583], current[1586], current[1587]
cell = sources[gen + 'model.rs']
plan = sources[gen + 'model/plan.rs']
layout = sources[gen + 'generator/layout.rs']
assert b'pub x: i16' in cell and b'pub z: i16' in cell and b'pub struct BuildingPlan' in plan
assert b'c.x - min_x + 1' in field and b'c.z - min_z + 1' in field and b'(width * depth * levels) as usize' in field
assert 32767 - 0 + 1 > 2**15 - 1
assert b'width > i16::MAX as u16' in layout and b'for x in 0..width' in layout and b'Cell::new(x as i16, z as i16)' in layout
assert b'for (field, building_transform) in fields' in exposure and b'field.sample(point)' in exposure
assert b'MAX_LIT_BUILDINGS: usize = 16' in lighting and b'MAX_GPU_SAMPLES: usize = 65_536' in lighting
assert b'nearby.truncate(MAX_LIT_BUILDINGS)' in lighting and b'entry.2.samples.len() <= MAX_GPU_SAMPLES' in lighting
assert b'camera.is_active' in lighting and b'distance <= FIELD_DISTANCE_METRES' in lighting
shader = sources['assets/shaders/tactical_interior_lighting.wgsl']
assert b'i < interior.counts.x' in shader and b'return vec4(0.0)' in shader and b'if sample.w != 0.0' in shader
windows = sources[gen + 'windows.rs']
assert b'plan.opening_assemblies' in windows and b'.map(|opening|' in windows and b'.find_map(|id|' in windows
assert b'for window in compile_operable_windows(plan)?' in sources['crates/adventuresim-tactical-server/src/windows.rs']
assert b'for leaf in compile_operable_windows(&building.plan)?' in sources[client + 'strategic_scene/instances.rs']
assert b'.filter(|(window, _)| window.building_id == building.id)' in shutters and b'.find(|entry| entry.0 == shutter.opening_id)' in shutters
assert b'InteriorField::from_plan(plan, local_origin.metres())' in sources[client + 'presentation/buildings.rs']
assert b'InteriorField::from_plan(' in sources[client + 'presentation/buildings/prepared.rs']

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
for n, indices in [(491,[1582]),(492,[1583,1584]),(493,[1585,1586,1587]),(494,[1588,1589])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
for n, indices in [(222,list(range(1574,1583))),(223,list(range(1583,1592)))]:
    path=f'adler/READ-L11-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
old = load('baseline-059/queue.json')['entries'][2244]
assert old['contract_id'] == 'L12-CLOSURE-SCENE-WINDOW-NATIVE-PROJECTION-LIGHTING'
refs = {}
def collect(value):
    if isinstance(value, dict):
        if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str): refs[value['path']] = value['sha256']
        if isinstance(value.get('registration'), str) and isinstance(value.get('registration_sha256'), str): refs[value['registration']] = value['registration_sha256']
        for nested in value.values(): collect(nested)
    elif isinstance(value, list):
        for nested in value: collect(nested)
collect(old)
assert all(path in records and lookup[path]['sha256'] == hash_ for path, hash_ in refs.items())
physical_receipt = json.loads(records['parallel-l12-20261029/READ-CONNECTED-PAR-L12-866-INTERIOR-FIELD-CONTRACT.json'])
assert physical_receipt['current_source_reconciled'] is False and physical_receipt['accepted_unique_count_change'] == 0
assert physical_receipt['rows'][0]['source_sha256'] == w['roles'][2556]['historical_sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1325,1326,1327,2244))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
p = load('CONTINUATION-PROGRESS-95E.json')
assert len(a['conditional_findings']) == 2
for item in a['conditional_findings']:
    assert item['repair'] == 'not started' and item['validation'] == 'not performed' and item['accepted_historical_unique_count_change'] == 0
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-INTERIOR-LIGHTING-95E.json' for x in ledger) == 1
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
check = load(baseline['latest_main_recheck'])
assert check['observed_revision'] == revision and check['ref'] == 'refs/heads/main'
print('Verified seven historical bodies, physical-snapshot records, source connections and conditional field cases; no product execution.')
