#!/usr/bin/env python3
"""Verify the room source trace and original evidence without product execution."""
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
name = 'ASSESSMENT-WEIGH-HOUSE-SIGNATURE-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1717-1719.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('WEIGH-HOUSE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1717:(438,440),1718:(288,288),1719:(941,941)}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        oldn,newn = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,oldn]]
        assert len(old.splitlines()) == oldn and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1717-1719.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == newn
        assert c['current_personal_ranges'] == [[1,newn]] and c['byte_identical'] == (old == body) == (i == 1718)
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if i in (1717,1719):
            assert old.split(b'#[cfg(test)]')[0] == body.split(b'#[cfg(test)]')[0]
            assert (root / c['diff']).read_bytes().startswith(b'diff --git ')
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
parent = load('ASSESSMENT-L01-HISTORICAL-1713-1716.json')['historical_source_roles'][1]['current_comparison']
parent_body = contents(parent['preserved_bytes']).decode()
uses = parent_body[parent_body.index('const ROOM_USES:'):parent_body.index('const ROOM_LABELS:')]
labels = parent_body[parent_body.index('const ROOM_LABELS:'):parent_body.index('pub(super) const VIEWS:')]
assert re.findall(r'BuildingUse::(\w+)',uses)[13] == 'WeighHouse'
assert re.findall(r'\("([^\"]+)",\s*"([^\"]+)"\)',labels)[13] == ('weigh-house','Weighing and line reckoning')
assert '13 => rooms::RoomSelection::Role(RoomKind::CountingRoom)' in parent_body
rooms = current[1717]
signature = rooms[rooms.index(b'fn signature_kind('):rooms.index(b'fn add_stair_subjects(')]
assert b'RoomKind::CountingRoom' not in signature and b'_ => false' in signature
assert b'if matches!(selection, RoomSelection::UpperKeep)' in rooms and b'add_stair_subjects(building, floor, &mut subjects)' in rooms
sightlines = current[1718]
assert b'let needs_signature = subjects.iter().any(|subject| subject.signature)' in sightlines
assert b'if needs_signature && !signature_visible' in sightlines
budget = sources['crates/adventuresim-building-generator/src/interior/budgets.rs']
section = budget[budget.index(b'RoomKind::CountingRoom =>'):budget.index(b'RoomKind::Guardroom')]
assert b'add(ReckoningTable,' in section and b'usage == BuildingUse::WeighHouse' in section and b'add(BalanceTable,' in section and b'add(Cupboard,' in section
assert b'TownHall | WeighHouse | CustomsHouse | Mint | AssayHouse' in sources['crates/adventuresim-building-generator/src/settlement.rs']
prior = load('ASSESSMENT-SCENE-CAPTURE-PERSISTENCE-95E.json')['current_manifest_source']
assert prior['original_index'] == 1719 and prior['historical_personal_full_read'] is False and prior['additional_original_frontier_credit'] == 0

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
for n, indices in [(551,[1717,1718]), (552,[1719])]:
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
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-WEIGH-HOUSE-SIGNATURE-OMISSION-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
q = load('RECONCILIATION-059.json')['entries']
assert q[257]['current_reconciliation'].startswith('partial ') and 'ASSESSMENT-L01-HISTORICAL-1717-1719.json' in q[257]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified room/manifest roles, prior evidence and counting-room source deduction; no product execution.')
