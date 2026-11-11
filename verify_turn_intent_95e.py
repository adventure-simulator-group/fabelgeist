#!/usr/bin/env python3
"""Verify pinned source evidence and exact bounds; no product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
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
name = 'ASSESSMENT-TURN-INTENT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1787-1789.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('TURN-INTENT-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1787:272,1788:145,1789:432}
ranges = {1787:[[1,272]],1788:[[9,22],[68,82],[118,145]],1789:[[11,98]]}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read(); c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert old == body and c['byte_identical'] is True
        assert c['current_personal_ranges'] == ranges[i] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
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
native = {}
manifest = load('native-evidence/rust-nightly-2026-07-13/MANIFEST.json')
items = {item['path']:item for item in manifest['files']}
for item in a['native_source_members']:
    raw = (root / item['path']).read_bytes()
    body = gzip.decompress(raw) if item['content_encoding'] == 'gzip' else raw
    assert sha(body) == item['sha256'] and len(body) == item['bytes']
    assert all(1 <= lo <= hi <= len(body.splitlines()) for lo, hi in item['ranges'])
    rel = item['path'].removeprefix('native-evidence/rust-nightly-2026-07-13/')
    assert items[rel]['sha256'] == item['sha256'] and items[rel]['bytes'] == item['bytes']
    native[rel] = body
f32 = native['library/core/src/num/f32.rs']
primitive = native['library/core/src/primitive_docs.rs.gz']
assert b'Returns the maximum of the two numbers, ignoring NaN' in f32
assert b'Returns the minimum of the two numbers, ignoring NaN' in f32
assert b'If exactly one of the arguments is NaN' in f32
assert b'the "binary32" type defined in IEEE 754-2008' in primitive
assert b'roundTiesToEven direction defined in IEEE 754-2008' in primitive
assert b'nightly-2026-07-13' in sources['rust-toolchain.toml']
turn = current[1789]; projection = current[1787]; raised = current[1788]
assert b'std::f32::consts::PI / duration.seconds()' in turn
assert b'(self.native_radians_per_second() * elapsed.seconds().max(0.0))' in turn
assert b'.min(std::f32::consts::PI)' in turn and b'.max(0.0)' in turn
assert b'if duration.is_zero()' in turn
elapsed = sources['crates/adventuresim-core/src/elapsed.rs']
positive = sources['crates/adventuresim-tactical-core/src/animation/measurement.rs']
assert b'pub const ZERO: Self = Self(0.0);' in elapsed and b'if value.is_zero()' in positive
assert b'body_turn_seconds_per_half_turn: 0.25' in sources['content/tactical/combat.yaml']
# Exact representability and overflow bounds; no f32 expression or product code is run.
assert Fraction(1,2**140) == 2**9 * Fraction(1,2**149)
maximum_f32 = (2**24 - 1) * 2**104
assert 2**140 > maximum_f32
assert Fraction(51,1000) - Fraction(5,100) > Fraction(1,2**24)
assert b'The authority replicates only observed motion' in raised
assert b'NonnegativeSpeed::from_metres_per_second(0.051)' in raised
assert b'NonnegativeSpeed::from_metres_per_second(0.05)' in sources['crates/adventuresim-tactical-core/src/animation/locomotion.rs']
assert b'requested_local_direction.and_then(|direction| direction.try_normalize())' in raised
assert b'intent.speed().max(MINIMUM_REQUESTED_GUARD_SPEED)' in raised
assert b'advance_raised_locomotion_intent(' in projection
assert b'.last_movement\n                    .map(|movement|' in sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'.replicate::<SkeletonState>()' in sources['crates/adventuresim-tactical-netcode/src/replication.rs']
assert b'One complete, authoritative raised-guard swing' in sources['crates/adventuresim-tactical-core/src/animation/state.rs']
assert b'let permitted_residual = body_facing_turn().tracking_residual' in sources['crates/adventuresim-tactical-client/src/animation_viewer/report/metrics.rs']

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(579,[1787,1788]),(580,[1789])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in (1459,1464,1465,1466,1467):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-ACTION-STATE-95E.json' in q[1459]['current_reconciliation']
findings = a['conditional_findings']
assert {f['id'] for f in findings} == {'REVIEW-PAUSED-FACING-SUBNORMAL-DURATION-001','REVIEW-GUARD-INTENT-AUTHORITY-DOC-001'}
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in findings:
    assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
    assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical roles, prior receipts, pinned Rust semantics and symbolic bounds; no product execution.')
