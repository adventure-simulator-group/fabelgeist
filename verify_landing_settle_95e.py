#!/usr/bin/env python3
"""Verify evidence identities and exact source-directed arithmetic, not product behavior."""
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
name = 'ASSESSMENT-LANDING-SETTLE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1763-1766.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('LANDING-SETTLE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1763:26,1764:138,1765:64,1766:61}
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
height = current[1764]
assert b'let rate = compression.metres() / elapsed.seconds();' in height
assert b'if !rate.is_finite()' in height and b'AnimationQuantity::HeightSpeed' in height
measurement = sources['crates/adventuresim-tactical-core/src/animation/measurement.rs']
assert b'if value.is_zero()' in measurement and b'pub struct PositiveAnimationDuration(AnimationElapsed)' in measurement
landing = sources['crates/adventuresim-tactical-core/src/combat_config/landing.rs']
assert b'pub recovery_seconds: PositiveAnimationDuration' in landing
assert b'recovery_seconds' not in landing[landing.index(b'impl AnimationLandingConfig'):]
profile = sources['crates/adventuresim-tactical-core/src/animation/locomotion.rs']
assert b'recovery_seconds: config.recovery_seconds.elapsed()' in profile
heightclient = sources['crates/adventuresim-tactical-client/src/animation/procedural/height.rs']
assert b'HeightSpeed::recovery(' in heightclient and b'.recovery_seconds' in heightclient
assert b'landing_compression_for_impact(profile, impact_speed)' in heightclient
assert b'if skeleton.is_posture_transitioning()' in heightclient
compliance = sources['crates/adventuresim-tactical-core/src/animation/response_parameters.rs']
assert b'compression.clamp(minimum.metres(), maximum.metres())' in compliance
sink = sources['crates/adventuresim-tactical-client/src/animation/procedural.rs']
assert b'current.advance_height(skeleton)' in sink and b'Rejected locomotion height sample' in sink
assert b'minimum_compression_metres: 0.04' in sources['content/tactical/combat.yaml']
assert b'recovery_seconds: 0.16' in sources['content/tactical/combat.yaml']
# Exactly representable binary32 values, compared as rational powers of two.
duration = Fraction(1, 2**140)
compression = Fraction(1, 2**5)
max_binary32 = (Fraction(2) - Fraction(1,2**23)) * 2**127
assert compression / duration == 2**135 > max_binary32
assert Fraction(4,100) > compression
shaping = sources['crates/adventuresim-tactical-core/src/animation/shaping_time.rs']
assert b'if !value.is_finite()' in shaping and b'Ok(Self(value))' in shaping
assert b'(progress.fraction() + elapsed.seconds() / self.0).min(1.0)' in shaping
config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'pub settle_step_seconds: ResponseShapingTime' in config
assert b'settle_step_seconds' not in config[config.index(b'impl TacticalCombatConfig {'):]
settle = current[1765]
assert b'Ok(Self(value.min(1.0)))' in settle and b'Ok(Self(value))' in settle
assert b'self.0 >= 1.0' in settle
planning = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/settle/planning.rs']
assert b'progress: SettleProgress::ZERO' in planning
clientsettle = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/settle.rs']
assert b'.settle_step_seconds' in clientsettle and b'.advance_settle(settle.progress, delta_seconds)?' in clientsettle
ik = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/mod.rs']
assert b'if !state_delta_seconds.is_zero()' in ik and b'let safe_settle_fallback' in ik
assert b'|| safe_settle_fallback' in ik and b'terminal_settle_contacts_are_rendered' in ik
clearance = sources['crates/adventuresim-tactical-client/src/animation/procedural/ik/terrain_contacts/foot_clearance.rs']
assert b'let progress = progress.fraction().clamp(0.0, 1.0);' in clearance
progress = min(Fraction(0) + Fraction(1,2) / Fraction(-1), Fraction(1))
assert progress == Fraction(-1,2) and max(Fraction(0), min(progress, Fraction(1))) == 0 and progress < 1
assert b'settle_step_seconds: 0.28' in sources['content/tactical/combat.yaml']

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
for n, indices in [(570,[1763,1764,1765,1766]), (575,[1779,1780])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
        assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
        assert x['fresh_personal_body_reads_this_batch'] == []
q = load('RECONCILIATION-059.json')['entries']
for i in (842,847,1448,1455):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ids = {f['id'] for f in a['conditional_findings']}
assert ids == {'REVIEW-LANDING-RECOVERY-RATE-OVERFLOW-001','REVIEW-SETTLE-SHAPING-SIGN-POLICY-001'}
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
    assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
assert load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified four historical roles, prior receipts and two conditional arithmetic traces; no product execution.')
