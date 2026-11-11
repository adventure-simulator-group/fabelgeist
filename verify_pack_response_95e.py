#!/usr/bin/env python3
"""Verify source identities, historical receipts and symbolic ratio; no product runs."""
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
name = 'ASSESSMENT-PACK-RESPONSE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1770-1777.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('PACK-RESPONSE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1770:77,1771:194,1772:52,1773:95,1774:217,1775:86,1776:110,1777:207}
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
        same = i != 1777
        assert (old == body) == c['byte_identical'] == same
        assert c['current_personal_ranges'] == ([] if same else [[1,212]])
        assert c['current_content_read_via_identical_historical_ranges'] == ([[1,n]] if same else [])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
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
pack = current[1771]
assert b'A pack that owns any attack owns the complete attack choice' in pack
assert b'if main.is_none() && (attacks.swing || attacks.thrust)' in pack
assert b'offhand |= attacks.offhand;' in pack and b'if main.is_some() && offhand' in pack
assert b'pub fn validate_structure(&self)' in pack and b'pub fn validate_complete(' in pack
testsource = sources['crates/adventuresim-tactical-core/src/animation/tests.rs']
test = testsource[testsource.index(b'fn child_attack_set_is_all_or_nothing_and_empty_child_inherits()'):]
assert b'SemanticPose::AttackOffhand' in test and b'[SemanticPose::AttackSwing]' in test
assert b'library.attack_animations(&SWORD_PACK.clone())' in test and b'offhand: true' in test
loading = sources['crates/adventuresim-tactical-client/src/animation/loading.rs']
assert b'fn rebuild_loaded_library' in loading and b'clips: available' in loading
route = sources['crates/adventuresim-tactical-client/src/animation/semantic_route.rs']
assert b'.attack_animations(&resolved.state.animation_pack)' in route
reference = current[1774]
assert b'checked_nonnegative(value, AnimationQuantity::ReferenceAcceleration)?' in reference
assert b'acceleration / self.0' in reference
config = sources['crates/adventuresim-tactical-core/src/combat_config/procedural.rs']
assert b'forward_reference_acceleration_metres_per_second_squared: ReferenceAcceleration' in config
assert b'reference_acceleration' not in config[config.index(b'impl ProceduralAnimationConfig'):]
target = sources['crates/adventuresim-tactical-client/src/animation/procedural/body_response/target.rs']
assert b'.component_ratio(acceleration.z)' in target and b'.clamped_positive(' in target
body = sources['crates/adventuresim-tactical-client/src/animation/procedural/body_values.rs']
assert b'Self::from_radians(scale * angle.radians())' in body and b'AnimationQuantity::BodyAngle' in body
apply = sources['crates/adventuresim-tactical-client/src/animation/procedural/body_response.rs']
assert b'Rejected locomotion body response' in apply and b'.advance_for_presentation(' in apply
assert b'skeleton.is_posture_transitioning()' in apply and b'!skeleton.is_grounded()' in apply
settings = sources['content/tactical/combat.yaml']
assert b'forward_reference_acceleration_metres_per_second_squared: 12.0' in settings
assert b'lateral_reference_acceleration_metres_per_second_squared: 10.0' in settings
assert b'forward_acceleration_lean_degrees: 10.0' in settings
ratio = Fraction(1,2) / Fraction(1,2**140)
max_binary32 = (Fraction(2) - Fraction(1,2**23)) * 2**127
assert ratio == 2**139 > max_binary32

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
for n, indices in [(572,[1770,1771,1772,1773]), (573,[1774,1775,1776,1777])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in (1450,1451,1452):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings = a['conditional_findings']
assert {f['id'] for f in findings} == {'REVIEW-PACK-ATTACK-INHERITANCE-DOC-001','REVIEW-BODY-REFERENCE-ACCELERATION-RATIO-001'}
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in findings:
    assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
    assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
assert load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified eight historical roles, prior receipts and qualified pack/ratio traces; no product execution.')
