#!/usr/bin/env python3
"""Verify wheel evidence and symbolic arithmetic without product execution."""
import gzip
import hashlib
import json
import math
import struct
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
pred = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
name = 'ASSESSMENT-WHEEL-CUTOFF-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1745-1749.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('WHEEL-CURRENT-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1745:1039,1746:140,1747:208,1748:17,1749:44}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
oldbodies, current = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if i == 1746:
            assert c['status'] == 'absent from current Git tree' and c['path'] not in tree
        else:
            body = contents(c['preserved_bytes'])
            assert c['status'] == 'present' and c['byte_identical'] == (old == body) == (i in (1748,1749))
            assert c['current_personal_ranges'] == ([] if i in (1748,1749) else [[1,len(body.splitlines())]])
            assert c['current_content_read_via_identical_historical_ranges'] == ([[1,n]] if i in (1748,1749) else [])
            assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
            assert oid('blob', body) == c['git_blob'] == tree[c['path']]
            if i in (1745,1747):
                d = c['exact_diff']; raw = contents(d['preserved_bytes'])
                assert sha(raw) == d['sha256'] and len(raw) == d['bytes'] and raw.startswith(b'diff --git ')
            current[i] = body
        oldbodies[i] = old
prior = load(a['prior_predecessor_assessment'])
assert prior['contract_id'] == 'NEW-WHEEL-QUANTIZED-CUTOFF-001' and prior['current_revision'] == pred and prior['lens_assessed'] == 1
prior_sources = {x['path']:x for x in prior['current_sources']}
selected_sources = {x['path']:x for x in prior['reported_selected_sources']}
assert selected_sources['crates/adventuresim-tactical-client/src/ui/combat_state.rs']['sha256'] == sha(oldbodies[1746])
assert selected_sources['crates/adventuresim-tactical-client/src/ui.rs']['sha256'] != sha(oldbodies[1745])
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and item['predecessor_revision'] == pred and item['byte_identical_to_predecessor'] is True
    assert sha(body) == item['sha256'] == prior_sources[item['path']]['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']] == prior_sources[item['path']]['git_blob']
    sources[item['path']] = body
wheel = sources['crates/adventuresim-tactical-client/src/ui/wheel.rs']
page = sources['crates/adventuresim-stdb-module/static/tactical-hud.js']
core = sources['crates/adventuresim-tactical-core/src/player/combat_state.rs']
assert b'remaining -= amount;' in wheel and b'(amount * ARC_QUANTUM).round() / ARC_QUANTUM' in wheel
assert b'const ARC_QUANTUM: f32 = 1000.0;' in wheel and b'const MIN_VISIBLE_INCAPACITATION_SEGMENT: f32 = 0.005;' in wheel
assert b'Self::Fatigue => Color::srgb_u8(0x20, 0x20, 0x20)' in wheel and b'Self::Encumbrance => Color::srgba_u8(160, 160, 160, 120)' in wheel and b'Self::Projected => Color::srgb_u8(0xff, 0xff, 0x00)' in wheel
assert b'const end = angle + arc.share * Math.PI * 2;' in page and b'angle = end;' in page
assert b'state.incapacitation_sources(limbs.total_damage(), will)' in wheel and b'pub projected_increase: TacticalIncapacitationSources' in core
assert b'let end = cursor + amount * std::f32::consts::TAU;' in oldbodies[1745]
assert b'available = (1.0 - current.total()).clamp(0.0, 1.0)' in oldbodies[1746]
# Evidence-only binary32 arithmetic; no Rust/JS product function is invoked.
def f32(value):
    return struct.unpack('!f', struct.pack('!f', value))[0]
remaining, shares = f32(1.0), []
for value in (f32(0.3336), f32(0.3336), f32(0.3328)):
    amount = min(value, remaining)
    remaining = f32(remaining - amount)
    scaled = f32(amount * f32(1000.0))
    shares.append(f32(f32(math.floor(scaled + 0.5)) / f32(1000.0)))
assert all(value >= 0.005 for value in shares) and sum(shares) > 1.0009

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
for n, indices in [(564,[1745]), (565,[1746,1747,1748,1749])]:
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
    for key in ('qualified_supporting_same_lens_reuse','qualified_supporting_native_same_lens_reuse'):
        for x in receipt.get(key, []):
            assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
            assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
    if n == 564: assert len(receipt['partial_sequence']) == 2
    if n == 565: assert len(receipt['qualified_supporting_same_lens_reuse']) == 2
finding = a['existing_finding_connections'][0]
assert finding['id'] == prior['contract_id'] and finding['lens'] == finding['historical_lens_assessed'] == 1
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
predecessor = json.loads(contents('main-advance-95e/predecessor-baseline.json.gz'))
assert predecessor['repair_ledger']['entries'][31]['id'] == finding['id']
q = load('RECONCILIATION-059.json')['entries']
for i in (1442,1443):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name and item['historical_lens_assessed'] == 1 for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified historical UI/audio, current wheel and predecessor arithmetic; no product execution.')
