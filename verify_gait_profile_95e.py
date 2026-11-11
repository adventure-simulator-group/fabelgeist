#!/usr/bin/env python3
"""Verify Git-bound gait evidence and symbolic endpoint arithmetic only."""
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
name = 'ASSESSMENT-GAIT-PROFILE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1767-1769.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('GAIT-PROFILE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1767:267,1768:100,1769:90}
current_lengths = {1767:260,1768:107,1769:90}
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
        same = i == 1769
        assert (old == body) == c['byte_identical'] == same
        assert c['current_personal_ranges'] == ([] if same else [[1,current_lengths[i]]])
        assert c['current_content_read_via_identical_historical_ranges'] == ([[1,n]] if same else [])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == current_lengths[i]
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
profile = current[1767]
assert b'pub reference_speed: NonnegativeSpeed' in profile and b'pub recovery_seconds: AnimationElapsed' in profile
assert b'reference_speed: config.reference_speed_metres_per_second.into()' in profile
assert b'recovery_seconds: config.recovery_seconds.elapsed()' in profile
assert b'let phase = phase.cycles();' in profile and b'delta.min(1.0 - delta)' in profile
assert b'profile.support_phase_radius.cycles() * 0.45' in profile
assert b'profile.step_distance.metres().max(0.01)' in profile
config = sources['crates/adventuresim-tactical-core/src/combat_config/locomotion.rs']
assert b'pub reference_speed_metres_per_second: PositiveSpeed' in config
assert b'self.run.reference_speed_metres_per_second <= self.walk.reference_speed_metres_per_second' in config
landing = sources['crates/adventuresim-tactical-core/src/combat_config/landing.rs']
assert b'pub recovery_seconds: PositiveAnimationDuration' in landing
values = sources['crates/adventuresim-tactical-core/src/animation/sample_values.rs']
assert b'Finite cycles, deliberately signed and unwrapped until a periodic operation' in values
assert b'Self(self.0.rem_euclid(1.0))' in values
state = sources['crates/adventuresim-tactical-core/src/animation/state.rs']
assert b'pub gait_phase: CyclePhase' in state and b'gait_phase: wire.gait_phase.wrapped()' in state
assert b'self.gait_phase = phase.wrapped()' in state
projection = sources['crates/adventuresim-tactical-core/src/animation/state/projection.rs']
assert b'skeleton.gait_phase = next_phase.wrapped()' in projection
ik = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/locomotion.rs']
assert b'gait_support_weights(profile, skeleton.gait_phase)?' in ik
ik2 = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/mod.rs']
assert b'gait_support_weights(profile, skeleton.gait_phase.wrapped())' in ik2
evalsrc = sources['crates/adventuresim-tactical-core/src/animation/evaluation/locomotion.rs']
assert b'phase: phase.wrapped()' in evalsrc
assert b'support_phase_radius: 0.175' in sources['content/tactical/combat.yaml']
# The example stays outside the smoothstep transition: no simulated product code.
radius = Fraction(7,40)
edge0 = radius * Fraction(45,100)
phase1, phase0 = Fraction(5,4), Fraction(1,4)
def distance(phase, contact):
    delta = abs(phase - contact)
    return min(delta, 1-delta)
assert (distance(phase1,0),distance(phase1,Fraction(1,2))) == (Fraction(-1,4),Fraction(1,4))
assert (distance(phase0,0),distance(phase0,Fraction(1,2))) == (Fraction(1,4),Fraction(1,4))
assert Fraction(-1,4) < edge0 < radius < Fraction(1,4)

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
receipt = json.loads(records['segovb/READ-L12-571.json'])
check = json.loads(records['segovb/VERIFY-L12-571.json'])
assert [row['original_index'] for row in receipt['rows']] == [1767,1768,1769]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
assert check['receipt_sha256'] == lookup['segovb/READ-L12-571.json']['sha256']
assert check['state_sha256'] == lookup['segovb/STATE-L12-571.json']['sha256']
for row in receipt['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
for x in receipt['qualified_supporting_same_lens_reuse']:
    assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
    assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
    assert x['fresh_personal_body_reads_this_batch'] == []
def adpath(value):
    return 'adler/' + value.replace('\\','/').rsplit('/',1)[-1]
for n in (259,282,283):
    path = f'adler/READ-L11-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in receipt['sources']:
        p = adpath(row['registration'])
        assert p in records and row['registration_sha256'] == lookup[p]['sha256']
    for x in receipt['same_lens_context']:
        p = adpath(x['receipt'])
        assert p in records and x['sha256'] == lookup[p]['sha256']
assert 'notactualconfiguredzero' in json.dumps(json.loads(records['adler/READ-L11-283.json']))
q = load('RECONCILIATION-059.json')['entries']
for i in (97,845,1449):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings = a['existing_finding_connections'] + a['conditional_findings']
assert {f['id'] for f in findings} == {'ADLER-L11-LOCOMOTION-PROFILE-ADMISSION','REVIEW-GAIT-SUPPORT-PERIODICITY-001'}
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in findings:
    assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
    assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
assert load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical roles, prior Adler/Segovb receipts and source-directed periodicity example; no product execution.')
