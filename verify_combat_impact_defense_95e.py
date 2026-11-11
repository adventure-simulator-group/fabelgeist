#!/usr/bin/env python3
"""Verify impact and defense source evidence without running the product."""
import gzip, hashlib, json, tarfile, tomllib
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
def verify_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for step in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == step['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
pr = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-COMBAT-IMPACT-DEFENSE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2033-2034.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][pr]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == pr and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2033: 161, 2034: 255}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed', 'bounded-current-trace-only') and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1, n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        verify_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert all((root / p).is_file() for p in a['prior_current_evidence'])
def walk(x):
    if isinstance(x, dict):
        yield x
        for value in x.values(): yield from walk(value)
    elif isinstance(x, list):
        for value in x: yield from walk(value)
for item in h['historical_source_roles']:
    for prior in item['preexisting_current_support_not_indexed_in_prior_role_row']:
        record = load(prior['assessment'])
        assert any(x.get('path') == item['current_comparison']['path'] and x.get('revision') == rev and x.get('ranges') == prior['ranges'] for x in walk(record)), (item['original_index'], prior)
contact, defense = current[2033], current[2034]
config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
startup = sources['crates/adventuresim-tactical-server/src/startup.rs']
server = sources['crates/adventuresim-tactical-server/src/combat.rs']
melee = sources['crates/adventuresim-tactical-server/src/combat/melee.rs']
active = sources['crates/adventuresim-tactical-server/src/combat/melee/active_defense.rs']
client = sources['crates/adventuresim-tactical-client/src/combat_effects.rs']
message = sources['crates/adventuresim-tactical-netcode/src/message.rs']
unit = sources['crates/adventuresim-core/src/unit_interval.rs']
assert b'body_part_hitboxes.len() != 7' in config and b'if seen[index]' in config
assert b'value.is_finite() && value > 0.0' in config
assert b'.validate()' in startup and b'install_runtime_snapshot()' in startup
assert b'.compute_affine()' in contact and b'.inverse()' in contact and b'.transform_point3(attacker_position)' in contact
assert b'.normalize_or(Vec3::Z)' in contact and contact.count(b'.is_finite()') == 3
assert b'.fold(f32::INFINITY, f32::min)' in contact and b'let point = center + direction * scale;' in contact
assert b'canonical_impact_surface(attacker_position, target_transform, body_part, config)' in server
assert b"Canonical impact position in the hit character's local space" in message
assert b"Canonical outward surface normal in the hit character's local space" in message
assert b'target_transform.transform_point(event.impact_point)' in client
assert b'.transform_vector3(event.impact_normal)' in client
for path in ('crates/adventuresim-tactical-server/src/combat/melee/outcome.rs', 'crates/adventuresim-tactical-server/src/combat/ranged/outcome.rs'):
    assert b'authoritative_impact(' in sources[path] and b'impact_point' in sources[path] and b'impact_normal' in sources[path]
assert b'if reflex_window.is_zero()' in defense and b'if window == 0.0' in defense
assert melee.index(b'prepare_active_defense(') < melee.index(b'completion.authorize(') < melee.index(b'commit_active_defense(')
assert b'authority.preserve_attack_for_trade();' in defense and b'prepared.commit(authority.as_deref_mut())' in active
assert b'if !value.is_finite()' in unit and b'if !(0.0..=1.0).contains(&value)' in unit
assert b'.expect("incapacitation preserves finite bounded precision")' in melee
# Bind the inspected normalization source to the exact lock-pinned archive.
native = a['native_source_evidence']; manifest = load(native['manifest']); package = root / native['archive']
raw = package.read_bytes(); assert sha(raw) == manifest['sha256'] and len(raw) == manifest['bytes']
lock = contents('main-advance-95e/tree-stream-evidence/current-cargo-lock.source.gz')
assert oid('blob', lock) == tree['Cargo.lock']
packages = tomllib.loads(lock.decode())['package']
math = next(x for x in packages if x['name'] == 'bevy_math' and x['version'] == '0.19.1')
glam = next(x for x in packages if x['name'] == 'glam' and x['version'] == '0.32.1')
assert 'glam 0.32.1' in math['dependencies'] and glam['checksum'] == manifest['sha256']
with tarfile.open(package, 'r:gz') as ar:
    body = ar.extractfile(native['member']).read()
    item = next(x for x in manifest['inspected_members'] if x['path'] == native['member'])
    assert sha(body) == native['sha256'] == item['sha256'] and len(body) == native['bytes'] == item['bytes']
    assert all(1 <= lo <= hi <= len(body.splitlines()) for lo, hi in native['personally_inspected_ranges'])
    assert b'pub fn normalize_or(self, fallback: Self)' in body and b'let rcp = self.length_recip();' in body
    assert b'if rcp.is_finite() && rcp > 0.0' in body and b'self * rcp' in body
# Source records are checked against the immutable original transport.
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else: assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob']
    records[item['original_path']] = body
for n, indices in ((682, [2033, 2034]), (686, [2040, 2041])):
    path = f'segovb/READ-L12-{n}.json'; data = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in data['rows']] == indices and data['current_source_reconciled'] is False
    assert data['plan'] in records and data['plan_sha256'] == lookup[data['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in data['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        if n == 682: assert row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]]
inherited = json.loads(records['segovb/READ-L12-686.json'])['connected_dispositions'][0]['qualified_inherited_same_lens_reuse']
assert inherited['original_index'] == 1022 and inherited['fresh_personal_body_reads_this_batch'] == [] and inherited['frozen_git_revision'] == pr
assert inherited['receipt_sha256'] == lookup[inherited['receipt']]['sha256'] and inherited['registration_sha256'] == lookup[inherited['prospective_registration']]['sha256']
assert inherited['incoming_adopted_state_sha256'] == lookup[inherited['incoming_adopted_state']]['sha256']
inherited_receipt = json.loads(records[inherited['receipt']])['sources'][2]
registration = json.loads(records[inherited['prospective_registration']].splitlines()[inherited['registration_line_1based'] - 1])
for x in (inherited_receipt, registration):
    assert x['original_index'] == 1022 and x['source_id'] == inherited['source_id'] and x['sha256'] == inherited['source_sha256']
assert inherited_receipt['actual_ranges'] == [[1, 58]] and registration['prospective_ranges'] == [[1, 58]]
adler = json.loads(records['adler/READ-L11-315.json'])
assert [x['original_index'] for x in adler['sources']] == [2029, 2030, 2031, 2032, 2033, 2034] and adler['current_reconciled'] is False
q = load('RECONCILIATION-059.json')['entries']; baseline_q = load('baseline-059/queue.json')['entries']
for i in (271, 1715, 1716, 1717, 1718):
    assert q[i]['contract_id'] == baseline_q[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    row = next(x for x in ledger if x['id'] == finding['id'])
    assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed'
assert a['conditional_findings'][0]['historical'] == 'baseline-059/ledger.json#/discovery_findings_and_dependencies/SEGOVB-L12-REPORTED-PRECISION-CLOSED-SCALING-PANIC'
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified two historical contact/defense roles, lock-pinned vector source, original inherited attribution and bounded impact paths; no product execution.')
