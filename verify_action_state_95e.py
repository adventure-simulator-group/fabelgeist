#!/usr/bin/env python3
"""Verify archived source identities and exact symbolic arithmetic, never product code."""
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
name = 'ASSESSMENT-ACTION-STATE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1781-1786.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('ACTION-STATE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1781:(2069,2067),1782:(89,89),1783:(353,353),1784:(153,153),1785:(123,123),1786:(158,160)}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n, m = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        same = i in (1782,1783,1784,1785)
        assert (old == body) == c['byte_identical'] == same
        assert c['current_personal_ranges'] == ([[1,m]] if i in (1781,1782,1786) else [])
        assert c['current_content_read_via_identical_historical_ranges'] == ([[1,n]] if same else [])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == m
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if not same:
            diff = contents(c['exact_diff']['preserved_bytes'])
            assert sha(diff) == c['exact_diff']['sha256'] and len(diff) == c['exact_diff']['bytes']
            assert b'diff --git ' in diff and b'@@ ' in diff
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
state, timeline = current[1781], current[1782]
client = sources['crates/adventuresim-tactical-client/src/player/attack_start.rs']
clock = sources['crates/adventuresim-tactical-client/src/player/action_clock.rs']
curve_config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
tangent = sources['crates/adventuresim-tactical-core/src/animation/continuation_parameters.rs']
assert b'continuation: false,' in state and b'let spec = AttackSpec { curve, ..default() };' in client
assert b'&& spec.continuation' in state and b'sequence: if spec.continuation {' in state
assert b'curve.overshoot\n                        * queued_timeline.preparation_ticks.samples() as f32' in state
assert b'IncomingAttackTangent::from_arithmetic(' in state and b'AnimationQuantity::ContinuationTangent' in tangent
assert b'Err(ActionTransitionError::InvalidAnimationValue(error)) => return Err(error)' in client
assert b'.begin_attack_timed(spec, timing.start, timing.contact, timing.end)\n                .is_err()' in client
assert b'skeleton.attack_continuation_tick().unwrap_or(input)' in clock
assert b'pub maximum_overshoot: f32' in curve_config and b'curve.maximum_overshoot,' in curve_config
assert b'.max(AnimationSampleSpan::ONE)' in timeline and b'if current_tick >= end_tick' in timeline
assert b'let mut action = self.action;' in state and b'match timeline.sample(current_tick)?' in state
assert b'overshoot_base: 0.08' in sources['content/tactical/combat.yaml']
assert b'maximum_overshoot: 0.55' in sources['content/tactical/combat.yaml']
# Exact source-directed illustration, not a run of animation code or f32 evaluation.
maximum_f32 = (2**24 - 1) * 2**104
assert 2**127 <= maximum_f32 < 2**128 and 2**127 * 4 > maximum_f32
assert 10 + (10 * Fraction(1,2)).__ceil__() == 15
assert Fraction(55,100) * 2**64 < maximum_f32
config = sources['crates/adventuresim-tactical-core/src/combat_config/state_transitions.rs']
assert b'self.dive_root_handoff_start_fraction >= self.dive_airborne_root_handoff_end_fraction' in config
assert b'DiveTrajectory::GroundedSlide => UnitInterval::ONE' in current[1786]
assert b'DIVE_GROUND_CONTACT_PHASE: UnitInterval = UnitInterval::HALF' in sources['crates/adventuresim-tactical-core/src/animation/evaluation.rs']
assert b'guard.maximum_step_seconds < guard.minimum_step_seconds' in curve_config
assert b'guard.planning_reach_metres <= guard.contact_margin_metres' in curve_config
assert b'.min(AnimationElapsed::from_seconds(1.0)?)' in sources['crates/adventuresim-tactical-core/src/animation/state/projection.rs']
assert b'physics_rotation.0 = rotation;' in sources['crates/adventuresim-tactical-server/src/player_projection/posture_facing.rs']

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
for n, indices in [(454,[1499,1500]),(576,[1781]),(577,[1782,1783]),(578,[1784,1785,1786])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
def adpath(value):
    return 'adler/' + value.replace('\\','/').rsplit('/',1)[-1]
receipt = json.loads(records['adler/READ-L11-213.json']); check = json.loads(records['adler/VERIFY-L11-213.json'])
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False
assert check['receipt_sha256'] == lookup['adler/READ-L11-213.json']['sha256']
assert check['state_sha256'] == lookup['adler/STATE-L11-213.json']['sha256']
for row in receipt['sources']:
    p = adpath(row['registration']); assert p in records and row['registration_sha256'] == lookup[p]['sha256']
for row in receipt['same_lens_context']:
    p = adpath(row['receipt']); assert p in records and row['sha256'] == lookup[p]['sha256']
assert receipt['sources'][2]['sha256'] == sha(client)
original = load('baseline-059/queue.json')['entries'][63]
assert original['contract_id'] == 'ADLER-L11-ATTACK-TRANSITION-ERROR-ADMISSION'
assert original['exact_source_identities_and_assessed_ranges'][2]['revision_as_recorded'] is None
q = load('RECONCILIATION-059.json')['entries']
for i in (63,1457,1458,1459,1460,1461,1462,1463):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
finding = a['existing_finding_connections'][0]
assert finding['id'] == original['contract_id'] and finding['accepted_historical_unique_count_change'] == 0
assert finding['repair'] == 'not started' and finding['validation'] == 'not performed'
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0
assert progress['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified six historical roles, original Adler/Segovb records and conditional source arithmetic; no product execution.')
