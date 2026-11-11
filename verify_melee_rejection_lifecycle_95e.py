#!/usr/bin/env python3
"""Verify melee rejection lifecycle source and recovered evidence only."""
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
        end = body.index(b'\0', pos); mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex()); pos = end + 21
    assert pos == len(body)
    return result
def verify_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for step in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == step['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[step['name']] == (step['mode'], step['child']); current = step['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
pr = '6533e444fa37552927070cc8d8190907fd3b413e'
pre = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
name = 'ASSESSMENT-MELEE-REJECTION-LIFECYCLE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2040-2045.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence']); shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']: nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][pr]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == pr and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2040:473, 2041:186, 2042:129, 2043:86, 2044:325, 2045:105}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed', 'bounded-current-trace-only') and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        verify_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes); current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert all((root/p).is_file() for p in a['prior_current_evidence'])
def walk(x):
    if isinstance(x, dict):
        yield x
        for v in x.values(): yield from walk(v)
    elif isinstance(x, list):
        for v in x: yield from walk(v)
for item in h['historical_source_roles']:
    for prior in item['preexisting_current_and_predecessor_support_not_indexed_in_prior_role_row']:
        record = load(prior['assessment']); path = item['current_comparison']['path']; ranges = prior['ranges_as_recorded']
        if prior['source_revision'] == pre:
            assert 'c100 predecessor' in item['prior_worklist_observation_revision_qualification']
            assert record.get('revision', record.get('current_revision')) == pre
            assert any(x.get('path') == path and (x.get('ranges') == ranges or x.get('range') == ranges) for x in walk(record))
        else:
            assert prior['source_revision'] == rev
            assert any(x.get('path') == path and x.get('revision') == rev and x.get('ranges') == ranges for x in walk(record)), (item['original_index'],prior)
melee, active_defense, melee_outcome, protocol, ranged, ranged_outcome = (current[i] for i in lengths)
ingress = sources['crates/adventuresim-tactical-server/src/combat/ingress.rs']
authority = sources['crates/adventuresim-tactical-server/src/combat/authority.rs']
server_defense = sources['crates/adventuresim-tactical-server/src/combat/defense.rs']
core_defense = sources['crates/adventuresim-core/src/combat/defense.rs']
state = sources['crates/adventuresim-tactical-core/src/animation/state.rs']
timeline = sources['crates/adventuresim-tactical-core/src/animation/state/action_timeline.rs']
config = sources['content/tactical/combat.yaml']
pending = ingress[ingress.index(b'pub(crate) fn resolve_pending_melee_contacts'):ingress.index(b'pub(super) fn on_melee_action_request')]
assert pending.index(b'.remove::<PendingMeleeContact>()') < pending.index(b'commands.trigger(MeleeAttackIntent {')
assert b'authority.complete_miss();' in pending
prepare = melee[melee.index(b'fn prepare('):melee.index(b'fn authorize(')]
assert b'validate_melee_intent_cheap(facts)' in prepare and b'Err(reason)' in prepare and b'return None;' in prepare
assert b'complete_miss' not in prepare and b'authority.authorize_attack' not in prepare
assert b'MeleeIntentRejection::OutOfRange' in authority
assert b'let active = self.active.as_ref()?;' in authority and b'let windup = self.windup.as_ref()?;' in authority
assert b'active.started_at <= incoming_started_at' in authority and b'windup.ready_at.elapsed_since(now)' in authority
assert b'defender_skeleton.action_kind() == SkeletonAction::Attack' in server_defense
assert b'defender_view.shield_block_bonus() <= 0.0' in server_defense
assert b'PreparedMeleeDefense::FinishTrade' in server_defense
assert b'own_contact_after_incoming_seconds.seconds() <= 0.0' in core_defense and b'choice: CommittedThreatChoice::FinishTrade' in core_defense
assert b'ActionTimeline::with_recovery(start_tick, contact_tick, end_tick)' in state
assert b'ActionTimelineSample::Complete => action = ActionState::default()' in state
assert b'let end_tick = contact_tick.saturating_add(recovery);' in timeline and b'if current_tick >= end_tick' in timeline
assert b'reflex_window_seconds: 0.5' in config and b'sample_hz: 64.0' in config
# The two source-selected locations use the same sample for body part selection.
targeting = sources['crates/adventuresim-core/src/combat/targeting.rs']; composite = sources['crates/adventuresim-core/src/composite.rs']
assert b'weighted_body_part_and_local(contact_sample.clamp(0.0, 1.0_f32.next_down()), |_| 1.0)' in targeting
assert b'(body_part, targeted_coordinate)' in targeting and b'melee_contact_location(&defender.equipment, contact_sample, gap_targeting)' in composite
assert b'contact_sample,' in melee and b'self.contact.sample' in contents(load('ASSESSMENT-L01-HISTORICAL-2035-2039.json')['historical_source_roles'][1]['current_comparison']['preserved_bytes'])
assert b'ranged: false' in sources['crates/adventuresim-tactical-server/src/iteration/equipment.rs']
assert b'commands.trigger(ApplyMeleeAttackResult {' in ranged_outcome and b'contact_at_time: MeleeContactAtTime::intended(0.0)' in ranged_outcome
# Bind framework queue source to the current lock-pinned archive.
native_refs = a['native_source_evidence']; native = load(native_refs[0]['manifest']); archive = root/native_refs[0]['archive']; raw = archive.read_bytes()
assert sha(raw) == native['sha256'] and len(raw) == native['bytes']
lock = contents('main-advance-95e/tree-stream-evidence/current-cargo-lock.source.gz'); assert oid('blob', lock) == tree['Cargo.lock']
packages = tomllib.loads(lock.decode())['package']; bevy = next(x for x in packages if x['name'] == 'bevy_ecs' and x['version'] == '0.19.1')
assert bevy['checksum'] == native['sha256']; nativebody = {}
with tarfile.open(archive,'r:gz') as package:
    for ref in native_refs:
        body = package.extractfile(ref['member']).read(); assert sha(body) == ref['sha256'] and len(body) == ref['bytes']
        assert all(1 <= lo <= hi <= len(body.splitlines()) for lo,hi in ref['personally_inspected_ranges'])
        item = next((x for x in native['inspected_members'] if x['path'] == ref['member']), None)
        assert ref['previously_indexed_member'] == (item is not None)
        if item: assert sha(body) == item['sha256'] and len(body) == item['bytes']
        nativebody[ref['member']] = body
commands = nativebody['bevy_ecs-0.19.1/src/system/commands/mod.rs']; trigger = nativebody['bevy_ecs-0.19.1/src/system/commands/command.rs']; queue = nativebody['bevy_ecs-0.19.1/src/world/command_queue.rs']
assert b'self.queue(command::trigger(event));' in commands and b'self.queue_handled(entity_command::remove::<B>(), warn)' in commands
assert b'world.trigger_ref_with_caller(' in trigger and b'while local_cursor < stop' in queue
# Check original059 record bytes and original authors/qualified same-lens reuse.
transport = json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else: assert sha(body) == item['sha256'] and oid('blob',body) == item['git_blob']
    records[item['original_path']] = body
for n, indices in ((686,[2040,2041]),(687,[2042,2043,2044,2045])):
    path = f'segovb/READ-L12-{n}.json'; data = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in data['rows']] == indices and data['current_source_reconciled'] is False
    assert data['plan'] in records and data['plan_sha256'] == lookup[data['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in data['rows']:
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]] and row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for row in data.get('qualified_supporting_same_lens_reuse',[]):
        assert row['receipt'] in records and row['registration'] in records and row['fresh_personal_body_reads_this_batch'] == []
inherit = json.loads(records['segovb/READ-L12-686.json'])['connected_dispositions'][0]['qualified_inherited_same_lens_reuse']
assert inherit['original_index'] == 1022 and inherit['fresh_personal_body_reads_this_batch'] == [] and inherit['frozen_git_revision'] == pr
assert inherit['receipt_sha256'] == lookup[inherit['receipt']]['sha256'] and inherit['registration_sha256'] == lookup[inherit['prospective_registration']]['sha256']
for n, indices in ((317,[2040,2041,2042]),(318,[2043,2044,2045])):
    data = json.loads(records[f'adler/READ-L11-{n}.json']); assert [x['original_index'] for x in data['sources']] == indices and data['current_reconciled'] is False
q = load('RECONCILIATION-059.json')['entries']; baseline_q = load('baseline-059/queue.json')['entries']
for i in (1718,1720,1722,1723,1724,1725,1726,1731,1733,1735,1736,1737,1738):
    assert q[i]['contract_id'] == baseline_q[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']; finding = a['conditional_findings'][0]
row = next(x for x in ledger if x['id'] == finding['id']); assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed'
for ident in ('SEGOVB-L12-REPORTED-PRECISION-CLOSED-SCALING-PANIC','SEGOVB-L12-RANGED-MISS-HIT-PAYLOAD-OWNERSHIP'):
    assert name in next(x for x in ledger if x['id'] == ident)['additional_current_evidence']
p = load('CONTINUATION-PROGRESS-95E.json'); assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified six historical melee/ranged roles, predecessor attribution, pinned Bevy queue source and bounded rejection lifecycle; no product execution.')
