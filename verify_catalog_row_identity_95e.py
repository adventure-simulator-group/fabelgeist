#!/usr/bin/env python3
"""Verify source provenance and static catalogue row comparisons."""
import gzip
import hashlib
import json
import re
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
name = 'ASSESSMENT-CATALOG-ROW-IDENTITY-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1713-1716.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('CATALOG-ROW-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1713:241,1714:379,1715:201,1716:89}
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
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1713-1716.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
parent = current[1714].decode()
children = current[1715].decode()
finish = current[1716].decode()
furniture = sources['crates/adventuresim-building-generator/src/furniture.rs'].decode()
keys = sources['crates/adventuresim-building-generator/src/furniture/key.rs'].decode()
view = sources['crates/adventuresim-tactical-client/src/tactical_scene_viewer/view_camera.rs']
rows = re.findall(r'\("([a-z-]+)",\s*"([^"]+)"\)', parent[parent.index('const CATALOG:'):parent.index('const ROOM_USES:')])
part = furniture[furniture.index('pub const INTERIOR:'):furniture.index('pub const ALL:', furniture.index('pub const INTERIOR:'))]
kinds = re.findall(r'Self::(\w+)', part)
slug = lambda s: re.sub(r'(?<!^)(?=[A-Z])','-',s).lower()
part = keys[keys.index('pub const ALL:', keys.index('impl FinishableFurnitureKind')):keys.index('pub const fn kind', keys.index('impl FinishableFurnitureKind'))]
fkinds = re.findall(r'Self::(\w+)', part)
fslugs = re.findall(r'"finish-([a-z-]+)"', finish[:finish.index('const FINISH_INSTANCE')])
mapping = load(a['static_mapping'])
assert len(rows) == len(kinds) == len(mapping['interior_rows']) == 45
assert len(fkinds) == len(fslugs) == len(mapping['finish_rows']) == 5
for i,(kind,(slug_text,label)) in enumerate(zip(kinds,rows)):
    assert mapping['interior_rows'][i] == {'index':i,'kind':kind,'slug':slug_text,'label':label,'slug_matches_normalized_kind':slug(kind)==slug_text}
    assert slug(kind) == slug_text
for i,(kind,slug_text) in enumerate(zip(fkinds,fslugs)):
    assert mapping['finish_rows'][i] == {'index':i,'kind':kind,'slug':'finish-'+slug_text,'slug_matches_normalized_kind':slug(kind)==slug_text}
    assert slug(kind) == slug_text
variants = furniture[furniture.index('pub const ALL:',furniture.index('impl FurnitureVariant')):][:70]
states = keys[keys.index('pub const ALL:',keys.index('impl FurnitureWoodState')):][:100]
assert re.findall(r'Self::(\w+)',variants) == mapping['variant_order'] == ['Compact','Broad']
assert re.findall(r'Self::(\w+)',states) == mapping['wood_state_order'] == ['Natural','Handled','Repaired','Painted']
assert 'CATALOG[index].0' in parent and 'camera: index as u8' in parent
assert 'FurnitureKind::INTERIOR.into_iter().enumerate()' in children and 'cameras.push(catalog_camera(' in children
assert b'&state.city_exterior_cameras' in view and b'.get(usize::from(camera))' in view
assert mapping['interior_rows'][0]['kind'] == 'DiningTable' and mapping['interior_rows'][1]['kind'] == 'Bench'

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
for n, indices in [(549,[1713,1714]), (550,[1715,1716])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-FURNITURE-CATALOG-ROW-ASSOCIATION-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified four historical roles, original reviewer evidence and current catalogue row comparison; no product execution.')
