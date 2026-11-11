#!/usr/bin/env python3
"""Verify retained evidence and a rational illustration, not product behavior."""
import gzip, hashlib, json, tarfile
from fractions import Fraction
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
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-PRONE-EFFORT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1888-1890.json'
a, h, worklist = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence']); shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1888:522, 1889:81, 1890:640}
ranges = {1888:[[1,523]], 1889:[[1,81]], 1890:[]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i == 1890 else 'unassessed')
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert (before != body if i == 1888 else before == body)
        assert c['byte_identical'] == (i != 1888) and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == ([] if i == 1888 else [[1, n]])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == (523 if i == 1888 else n)
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
diff = a['historical_current_diff']; raw_diff = (root / diff['path']).read_bytes()
assert sha(raw_diff) == diff['sha256'] and len(raw_diff) == diff['bytes']
assert b'-        let smooth_endurance = endurance * endurance * (3.0 - 2.0 * endurance);' in raw_diff
assert b'+        let smooth_endurance = UnitInterval::from_fraction(endurance)?.smoothstep()?;' in raw_diff
interval = sources['crates/adventuresim-core/src/unit_interval.rs']
assert b'Self::from_fraction(fraction * fraction * (3.0 - 2.0 * fraction))' in interval
assert b'if !(0.0..=1.0).contains(&value)' in interval
movement = current[1888]
assert b'if matches!(body, Some(BodyState::Prone | BodyState::Supine))' in movement
assert b'movement.x * TACTICAL_PRONE_LATERAL_SPEED_SCALE' in movement
assert b'let effort_speed = if body == crate::animation::BodyState::Prone' in movement
assert b'MovementPace::Sprint => sprint_speed' in movement
projection = sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'else if weapon_guard == WeaponGuardState::Raised {\n            Self::Aim' in projection
assert b'movement_intent.0 = validated.movement;' in projection
assert b'skeleton.body() == BodyState::Prone && posture.facing == CameraFacingIntent::Aim' in projection
assert b'accumulated_input.last_movement = None;' in projection
assert b'-movement.x * lateral_speed_scale' in projection
assert b'BodyState::Supine && movement.y < 0.0' in projection
assert b'if roll_motion.is_moving()' in projection
assert b'BodyMassKg::try_new(player.body_weight_kg)' in projection
assert b'fatigue: player.strategic_fatigue' in projection
state = sources['crates/adventuresim-tactical-core/src/animation/state.rs']
assert b'if !skeleton.body.is_downed() && skeleton.posture_transition.is_none()' in state
condition = sources['crates/adventuresim-tactical-server/src/combat/condition.rs']
assert b'movement_intent.as_deref().and_then(|intent| intent.0)' in condition
assert b'let effort_speed = tactical_movement_speed_for_pace(' in condition
assert b'skeleton.map_or(WeaponGuardState::Lowered, SkeletonState::weapon_guard)' in condition
assert b'let moving = movement.is_some_and(|movement| movement.length_squared() > 0.0)' in condition
assert b'if !active_action && !moving' in condition
fatigue = sources['crates/adventuresim-core/src/combat/fatigue.rs']
assert b'(aerobic * ratio * ratio + carry - aerobic).max(0.0) / parameters.capacity(endurance)' in fatigue
client = sources['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'MovementPace::Sprint' in client and b'weapon_guard: guard.desired' in client
assert b'pace: controls.pace' in client and b'downed_align: controls.downed_align' in client
assert b'Cardinal::wasd_keys()' in sources['crates/adventuresim-tactical-client/src/player.rs']
assert b'.before(AdventureSimulatorPhysicsSet::ApplyCharacterMotor)' in sources['crates/adventuresim-tactical-server/src/main.rs']
assert b'Update,\n                (\n                    update_tactical_combat_state' in sources['crates/adventuresim-tactical-server/src/combat.rs']
yaml = sources['content/tactical/combat.yaml']
assert b'aerobic_watts_per_endurance: 70.0' in yaml and b'capacity_joules_per_endurance: 16000.0' in yaml
assert b'prone_lateral_speed_scale: 0.375' in yaml
# Exact rational illustration only: this is not native float or product execution.
rate = Fraction(70) * ((Fraction(8) / Fraction(18,10))**2 - 1) / 16000
assert rate == Fraction(10633,129600) and Fraction(8,100) < rate < Fraction(9,100)
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
path = 'segovb/READ-L12-624.json'; z = json.loads(records[path]); check = json.loads(records['segovb/VERIFY-L12-624.json'])
assert [x['original_index'] for x in z['rows']] == [1889,1890] and z['current_source_reconciled'] is False
assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['segovb/STATE-L12-624.json']['sha256']
for row in z['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]] and row['same_lens_reuse'] == []
path = 'adler/READ-L11-288.json'; z = json.loads(records[path]); check = json.loads(records['adler/VERIFY-L11-288.json'])
assert [x['original_index'] for x in z['sources']] == list(range(1888,1895))
assert z['current_reconciled'] is False and z['accepted_unique_count_change'] == 0
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-288.json']['sha256']
for row in z['sources'] + z.get('connected_reinspection', []):
    registration = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert registration in records and row['registration_sha256'] == lookup[registration]['sha256']
for row in z['same_lens_context']:
    receipt = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
    assert receipt in records and row['sha256'] == lookup[receipt]['sha256']
prior = load('ASSESSMENT-QUICKSTEP-FORCE-95E.json')
for path in ('segovb/READ-L12-623.json','evidence/continuation-segovb/audit-resume-20261027/lens08/batch-015-originals1886-1893.json'):
    rec = next(x for x in prior['recovered_historical_records'] if x['original_path'] == path)
    assert sha(contents(rec['preserved_bytes'])) == rec['published_sha256'] and rec['sha256'] == lookup[path]['sha256']
q = load('RECONCILIATION-059.json')['entries']
for i in (1561,1562,1563):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f = a['conditional_findings'][0]
assert f['id'] == 'REVIEW-PRONE-AIM-RETAINED-EFFORT-001'
assert f['accepted_historical_unique_count_change'] == 0 and f['repair'] == 'not started' and f['validation'] == 'not performed'
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == f['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical bodies, attributed records and bounded prone-effort source evidence; no product execution.')
