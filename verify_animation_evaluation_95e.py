#!/usr/bin/env python3
"""Verify recovered animation evidence; do not execute product code."""
import gzip
import hashlib
import json
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
name = 'ASSESSMENT-ANIMATION-EVALUATION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1758-1762.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('ANIMATION-EVALUATION-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1758:591,1759:82,1760:327,1761:81,1762:14}
current_lengths = {1758:582,1759:82,1760:320,1761:81,1762:14}
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
        same = i in (1759,1761,1762)
        assert (old == body) == c['byte_identical'] == same
        assert c['current_personal_ranges'] == ([[1,current_lengths[i]]] if i in (1758,1759,1760) else [])
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
locomotion = current[1760]
assert b'walk_locomotion_profile().reference_speed' in locomotion and b'run_locomotion_profile().reference_speed' in locomotion
assert b'.clamped_inverse_lerp(' in locomotion
config = sources['crates/adventuresim-tactical-core/src/combat_config/locomotion.rs']
assert b'self.run.reference_speed_metres_per_second <= self.walk.reference_speed_metres_per_second' in config
speed = sources['crates/adventuresim-core/src/linear_speed.rs']
assert b'if lower >= upper {' in speed and b'SpeedError::NonIncreasingBounds' in speed
runtime = sources['crates/adventuresim-tactical-core/src/combat_config/runtime.rs']
assert b'self.validate()?;' in runtime and b'RUNTIME_CONFIG_VERSION.fetch_add(1, Ordering::Release)' in runtime
assert b'CACHED_ANIMATION_CONFIG' in runtime and b'pub fn runtime_animation_config()' in runtime
client = sources['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'snapshot.0.validate()' in client and b'snapshot.0.install_runtime_snapshot()' in client
route = sources['crates/adventuresim-tactical-client/src/animation/semantic_route.rs']
assert b'AnimationEvaluation::from_skeleton(skeleton)?' in route and b'remove::<SemanticRouteTrace>()' in route
span = current[1761]
assert b'NonZeroU64::new(events)' in span and b'(0..self.0).map(Self)' in span and b'span.0 <= self.0.get()' in span
playback = sources['crates/adventuresim-tactical-core/src/combat_config/playback.rs']
assert b'pub maximum_coalesced_events: CoalescedEventBudget' in playback
assert b'maximum_coalesced_events' not in playback[playback.index(b'impl AnimationPlaybackConfig'):]
events = sources['crates/adventuresim-tactical-client/src/animation/presentation/events.rs']
assert b'maximum_coalesced_events.contains(*delta)' in events and b'.offsets()' in events and b'.collect()' in events
assert b'maximum_coalesced_events: 8' in sources['content/tactical/combat.yaml']
native = {}
for item in a['native_archive_members']:
    raw = (root / item['archive']).read_bytes()
    assert sha(raw) == item['archive_sha256']
    with tarfile.open(root / item['archive'], 'r:gz') as archive:
        body = archive.extractfile(item['member']).read()
    assert sha(body) == item['member_sha256'] and len(body) == item['member_bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['personal_ranges'])
    native[item['member']] = body
replicon = native['bevy_replicon-0.41.1/src/shared/message/server_event.rs']
ecs_executor = native['bevy_ecs-0.19.1/src/schedule/executor/multi_threaded.rs']
ecs_commands = native['bevy_ecs-0.19.1/src/system/commands/mod.rs']
ecs_command = native['bevy_ecs-0.19.1/src/system/commands/command.rs']
assert b'commands.trigger(message.event);' in replicon and b'self.queue(command::trigger(event));' in ecs_commands
assert b'move |world: &mut World|' in ecs_command and b'world.trigger_ref_with_caller(' in ecs_command
assert b'Do one final apply buffers after all systems have completed' in ecs_executor
assert b'no other systems currently have access to the world' in ecs_executor and b'system.apply_deferred(world)' in ecs_executor
lock = sources['Cargo.lock']
for item in a['native_archive_members']:
    assert item['archive_sha256'].encode() in lock

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
for n, indices in [(568,[1758,1759]), (569,[1760,1761,1762])]:
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
for i in (99,822,1446,1447,1550,1624):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert not a['conditional_findings']
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'])
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical roles, prior receipts, bounded event/config source and native archive identities; no product execution.')
