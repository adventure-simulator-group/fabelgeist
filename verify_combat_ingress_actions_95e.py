#!/usr/bin/env python3
"""Verify ingress source and recovered attribution without executing product code."""
import gzip, hashlib, json, tarfile
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
pre = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
name = 'ASSESSMENT-COMBAT-INGRESS-ACTIONS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2035-2039.json'
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
lengths = {2035: 1210, 2036: 295, 2037: 298, 2038: 168, 2039: 50}
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
    for prior in item['preexisting_current_and_predecessor_support_not_indexed_in_prior_role_row']:
        record = load(prior['assessment']); path = item['current_comparison']['path']
        if prior['source_revision'] == pre:
            assert record['current_revision'] == pre and 'does not become a new95e personal read' in item['prior_worklist_observation_revision_qualification']
            assert any(x.get('path') == path and x.get('ranges') == prior['ranges'] and x.get('git_blob') == tree[path] for x in walk(record))
        else:
            assert prior['source_revision'] == rev and any(x.get('path') == path and x.get('revision') == rev and x.get('ranges') == prior['ranges'] for x in walk(record)), (item['original_index'], prior)
ingress, start, lunge, timing, work_source = (current[i] for i in lengths)
projection = sources['crates/adventuresim-tactical-server/src/player_projection.rs']
client = sources['crates/adventuresim-tactical-client/src/player.rs']
protocol = sources['crates/adventuresim-tactical-server/src/combat/protocol.rs']
message = sources['crates/adventuresim-tactical-netcode/src/message.rs']
replication = sources['crates/adventuresim-tactical-netcode/src/replication.rs']
authority = sources['crates/adventuresim-tactical-server/src/combat/authority.rs']
seed = sources['crates/fabelgeist-determinism/src/seed.rs']
stream = sources['crates/fabelgeist-determinism/src/stream.rs']
config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
defense = sources['crates/adventuresim-tactical-server/src/combat/defense.rs']
combat_state = sources['crates/adventuresim-tactical-core/src/player/combat_state.rs']
miss = ingress[ingress.index(b'RangedActionRequest::CompleteMiss'):ingress.index(b'RangedActionRequest::CompleteHit')]
assert b'target: None' in miss and b'body_part: BodyPart::Chest' in miss and b'ReportedPrecision::new(0.0).expect(' in miss
assert b'pub(crate) target: Option<Entity>' in protocol and b'pub(crate) body_part: BodyPart' in protocol
assert b'CompleteMiss,' in message and b'CompleteHit {' in message
assert b'ValidatedRangedImpact::Miss' in authority
assert b'ammunition.consume(' in sources['crates/adventuresim-tactical-server/src/combat/ranged.rs']
block = ingress[ingress.index(b'pub(crate) fn apply_defend_intent'):ingress.index(b'pub(crate) fn on_melee_attack_started')]
assert b'DodgeSpec::quickstep(direction).is_none()' in block
assert b'if skeleton.action_kind() == SkeletonAction::Dodge => true' in block
assert b'CombatActionWork::ExplosiveDodge' in block and b'state.charge_work(' in block
assert b'.insert(PendingDefenderResponse {' in block and b'set_at: CombatInstant::from_elapsed(&time)' in block
assert projection.index(b'let accepted = begin_authoritative_quickstep(') < projection.index(b'commands.trigger(crate::combat::DefendIntent {')
assert b'cmd.client_trigger(DefendRequest::Roll)' in client and b'.add_client_event::<DefendRequest>(Channel::Ordered)' in replication
assert b'let before = self.fatigue;' in combat_state and b'self.incapacitation + self.fatigue - before' in combat_state
assert b'pending.set_at < incoming_started_at' in defense
assert b'let word = observed_sample_word(time);' in timing and b'key: MeleeAttackKey::from(word)' in timing and b'sample: AnimationSampleTick::from(word)' in timing
assert b'&windup.attack_key.to_le_bytes()' in authority and b'&[&active.started_at.native_nanos().to_le_bytes()]' in authority
assert b'pub fn derive(root: &[u8], stream: StreamId, context: &[&[u8]])' in seed
assert b'Self(rand_xoshiro::SplitMix64::from_seed(seed.to_le_bytes()))' in stream
assert b'let Ok(state) = combat_states.get(attacker) else' in work_source and b'RecoveryTime::After(recovery)' in work_source
assert b'let Ok(mut state) = combat_states.get_mut(attacker) else' in work_source
assert b'TacticalCombatState {' in projection and b'combat_projection_state()' in projection
assert b'let Some(target) = event.target else' in start and b'MeleeLungeArrival::CannotArrive' in start
assert b'!self.targeting.reported_hit_precision.is_finite()' in config
assert b'.expect("configured melee precision is finite")' in ingress
# Preserve original059 identities, independent authors and prior scope qualifications.
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
for n, indices in ((683, [2035]), (684, [2036, 2037]), (685, [2038, 2039]), (687, [2042, 2043, 2044, 2045])):
    path = f'segovb/READ-L12-{n}.json'; data = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in data['rows']] == indices and data['current_source_reconciled'] is False
    assert data['plan'] in records and data['plan_sha256'] == lookup[data['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in data['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        if row['original_index'] in lengths: assert row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]]
    for row in data.get('qualified_supporting_same_lens_reuse', []):
        assert row['receipt'] in records and row['registration'] in records
        assert row['receipt_sha256'] == lookup[row['receipt']]['sha256'] and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
adler = json.loads(records['adler/READ-L11-316.json'])
assert [x['original_index'] for x in adler['sources']] == [2035, 2036, 2037, 2038, 2039] and adler['current_reconciled'] is False
for n in (45, 47):
    path = f'root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-{n:03d}.json'; data = json.loads(records[path])
    check = json.loads(records[f'root-l1-current-e275-20261029/VERIFY-ROOT-L1-CURRENT-{n:03d}.json'])
    assert data['registration'] in records and data['registration_sha256'] == lookup[data['registration']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L1-CURRENT-{n:03d}.json']['sha256']
    assert all('current-main:e2755a732c84e67296b5e3441a4259a84f461b2f:' in row['source_id'] for row in data['rows'])
q = load('RECONCILIATION-059.json')['entries']; baseline_q = load('baseline-059/queue.json')['entries']
for i in (272, 815, 821, 1717, 1721, 1728, 1729):
    assert q[i]['contract_id'] == baseline_q[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    row = next(x for x in ledger if x['id'] == finding['id'])
    assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed'
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical ingress roles, predecessor and original reviewer qualifications, and bounded action sources; no product execution.')
