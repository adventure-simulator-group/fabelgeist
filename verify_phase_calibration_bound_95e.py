#!/usr/bin/env python3
"""Verify calibration evidence and symbolic binary32 arithmetic, not product behavior."""
import gzip
import hashlib
import json
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
name = 'ASSESSMENT-PHASE-CALIBRATION-BOUND-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1750-1757.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('CALIBRATION-BOUND-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1750:239,1751:12,1752:135,1753:174,1754:57,1755:55,1756:112,1757:14}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert old == body and c['byte_identical'] is True and c['current_personal_ranges'] == []
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
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
interval = current[1754]
assert b'pub struct PhaseCalibrationIntervals(u32)' in interval
assert b'if value < 3 {' in interval and b'Ok(Self(value))' in interval
pose = sources['crates/adventuresim-tactical-core/src/combat_config/pose_buffer.rs']
assert b'pub authored_phase_calibration_intervals: PhaseCalibrationIntervals' in pose
assert b'pose.maximum_authored_foot_range_metres < pose.minimum_authored_foot_range_metres' in pose
assert b'authored_phase_calibration_intervals' not in pose[pose.index(b'impl PoseBufferConfig'):]
config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'animation.pose_buffer.validate()?;' in config
runtime = sources['crates/adventuresim-tactical-core/src/combat_config/runtime.rs']
assert b'self.validate()?;' in runtime and b'pub fn runtime_animation_config()' in runtime
startup = sources['crates/adventuresim-tactical-server/src/startup.rs']
assert b'combat.map_or_else(default_combat_config_path, Path::to_path_buf)' in startup and b'.install_runtime_snapshot()' in startup
client = sources['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'snapshot.0.validate()' in client and b'snapshot.0.install_runtime_snapshot()' in client
curve = sources['crates/adventuresim-tactical-client/src/animation/pose_buffer/phase_calibration/curve.rs']
assert b'(0..=intervals)' in curve and b'index as f32 / intervals as f32' in curve
assert b'.collect::<AnimationValueResult<Vec<_>>>()?' in curve and b'smooth_periodic_phase_curve(&mut authored_phases, orientation)?' in curve
assert b'authored_phase_calibration_intervals: 256' in sources['content/tactical/combat.yaml']
def f32(value):
    return struct.unpack('!f', struct.pack('!f', value))[0]
n = 2 ** 24 + 1
assert n + 1 == 16_777_218 and f32(n) == f32(n-1) == 16_777_216.0
assert f32(f32(n-1) / f32(n)) == f32(f32(n) / f32(n)) == 1.0
tangent = current[1756]
assert b'if delta < 0.0 {' in tangent and b'IncomingAttackTangent::from_coordinate_delta(value.max(0.0))' in tangent
state = sources['crates/adventuresim-tactical-core/src/animation/state.rs']
evaluation = sources['crates/adventuresim-tactical-core/src/animation/evaluation.rs']
assert state.count(b'continuation_parameters::deserialize_tangent') == 2 and b'continuation_parameters::deserialize_tangent' in evaluation
assert b'IncomingAttackTangent::from_arithmetic(' in state

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
for n, indices in [(566,[1750,1751,1752,1753]), (567,[1754,1755,1756,1757])]:
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
    if n == 567:
        row = next(x for x in receipt['rows'] if x['original_index'] == 1756)
        assert 'notautomaticallyBCdefect' in json.dumps(row['function_contract'])
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-PHASE-CALIBRATION-INTERVAL-BOUND-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
q = load('RECONCILIATION-059.json')['entries']
for i in (1444,1445):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified calibration roles, prior receipts and conditional count/precision reasoning; no product execution.')
