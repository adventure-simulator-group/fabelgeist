#!/usr/bin/env python3
"""Verify inventory source evidence and recovery provenance; never run product code."""
import gzip
import hashlib
import json
import tarfile
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
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
older = 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
name = 'ASSESSMENT-INVENTORY-CACHE-IDENTITY-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1877-1881.json'
a, h, worklist = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1877:862, 1878:53, 1879:76, 1880:130, 1881:17}
ranges = {1877:[[35,175],[190,735]],1878:[[1,53]],1879:[[1,76]],1880:[[1,67]],1881:[[1,17]]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i == 1877 else 'unassessed')
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
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
inv = current[1877]
assert b'pub item_of: Option<&\'static ItemOf>' in inv
assert b'pub slot: Option<&\'static EquipSlot>' in inv
assert b'.and_then(|inventory| inventory.holding_shield)' in inv
assert b'.and_then(|weapon| self.q_item.get(weapon).ok())' in inv
assert b'let Some(root) = world.get::<ItemOf>(ctx.entity)' in inv
assert b'if items.holding_shield == Some(ctx.entity)' in inv
assert b'.and_then(|item| item.shield)' in inv
chrono = current[1879]
assert b'self.0.wrapping_add(1)' in chrono and b'Self::INITIAL' not in chrono
assert b'pub const INITIAL: Self = Self(0)' in chrono
ordering = current[1880]
assert b'.get(usize::from(occupancy.requirement_index))' in ordering
assert b'requirement.location == location && requirement.channel == occupancy.channel' in ordering
server = sources['crates/adventuresim-tactical-server/src/equipment.rs']
drop = server[server.index(b'fn drop_hand('):server.index(b'fn item_box_center(')]
assert drop.index(b'.remove::<ItemOf>()') < drop.index(b'.remove::<EquipSlot>()')
assert b'physical.is_valid()' in drop and b'has_children(item, items)' in drop
projection = sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'TacticalInventoryItemId(item.inventory_item_id)' in projection
assert b'commands.queue(move |world: &mut World| rebuild_inventory_holding_cache(world, client))' in projection
assert b'commands.queue(move |world: &mut World| rebuild_inventory_holding_cache(world, orphan))' in projection
armor = sources['crates/adventuresim-tactical-core/src/inventory_armor.rs']
assert b'intersected: geometry.contains(sample)' in armor and b'selected: selected == Some(layer_index)' in armor
causal = sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/causal.rs']
auto = sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/causal/autoresolve.rs']
assert b'format!("{}:{:?}", layer.item_id, layer.material)' in causal
assert b'format!("{:?}:{:?}", layer.inventory_item_id, layer.material)' in auto
assert b'vec![ArmorLayerTelemetry {' in sources['crates/adventuresim-core/src/autoresolve/melee_round/commitment.rs']
assert b'armor_layers_intersected: BTreeMap<String, u64>' in sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/report.rs']
assert b'shield_side.and_then(shield_resting_coordinate)' in sources['crates/adventuresim-core/src/combat/defense.rs']
assert b'defender_equip.shield_block_bonus()' in sources['crates/adventuresim-core/src/combat/targeting.rs']
native = load(a['native_evidence']['manifest']); byname = {x['path']:x for x in native['inspected_members']}; nativebody = {}
with tarfile.open(root / a['native_evidence']['archive'], 'r:gz') as package:
    for path, ranges in a['native_evidence']['personally_inspected_members_and_ranges'].items():
        body = package.extractfile(path).read(); info = byname[path]
        assert sha(body) == info['sha256'] and len(body) == info['bytes']
        assert native['personally_inspected_ranges'][path] == ranges
        assert all(1 <= lo <= hi <= len(body.splitlines()) for lo,hi in ranges)
        nativebody[path] = body
relationship = nativebody['bevy_ecs-0.19.1/src/relationship/mod.rs']
assert b'relationship_target.collection_mut_risky().remove(entity)' in relationship
assert b'if relationship_target.len() == 0' in relationship
assert b'entity.remove::<Self::RelationshipTarget>()' in relationship
commands = nativebody['bevy_ecs-0.19.1/src/system/commands/mod.rs']
assert b'self.queue_handled(entity_command::remove::<B>(), warn)' in commands
assert b'self.commands.queue(command.with_entity(self.entity))' in commands
assert b'entity.remove_with_caller::<T>(caller)' in nativebody['bevy_ecs-0.19.1/src/system/commands/entity_command.rs']
world_mut = nativebody['bevy_ecs-0.19.1/src/world/entity_access/world_mut.rs']
assert b'BundleRemover::new::<T>(self.world, location.archetype_id, false)' in world_mut
assert b'pub fn remove_with_requires<T: Bundle>' in world_mut
queue = nativebody['bevy_ecs-0.19.1/src/world/command_queue.rs']
assert b'while local_cursor < stop' in queue and b'(meta.consume_command_and_get_size)(cmd, world, &mut local_cursor)' in queue
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(617,[1877]),(618,[1878,1879]),(619,[1880]),(620,[1881,1882,1883])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['same_lens_reuse'] == []
        if row['original_index'] in lengths:
            assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
adler = json.loads(records['adler/READ-L11-285.json'])
assert [x['original_index'] for x in adler['sources']] == list(range(1877,1882))
for row in adler['sources']:
    registration = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert registration in records and row['registration_sha256'] == lookup[registration]['sha256']
qualification = json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1880-LOCATOR.json'])
assert qualification['body_coverage'] == qualification['new_population_credit'] == 0
assert qualification['historical_locator_proof_body_not_read'] is True
assert qualification['actual_EOF'] == 130
op = load(a['older_774_git_membership_evidence']); os = load(op['shared_commit_and_tree_objects'])
onodes = {x['git_object']:x for x in os['objects'] if x['type']=='tree'}
for path in op['additional_shared_tree_objects']:
    onodes.update(load(path)['additional_tree_objects'])
onodes.update(op['additional_tree_objects'])
oc = next(x for x in os['objects'] if x['type']=='commit' and x['git_object']==older)
raw = contents(oc['preserved_bytes'])
assert oid('commit',raw)==older==op['revision'] and raw.splitlines()[0]==b'tree '+op['root_tree'].encode()
older_records = {}
for item in a['recovered_older_774_records']:
    path = item['path_in_774']; body = contents(item['preserved_bytes'])
    assert item['older_publication_revision'] == older and sha(body) == item['sha256'] and len(body)==item['bytes']
    check_chain(path,op['paths'][path],body,op['root_tree'],onodes)
    older_records[path] = body
base = 'handoff/774/adler-original/'
package = json.loads(older_records[base+'PACKAGE-MANIFEST.json'])
package_lookup = {x['path']:x for x in package['entries']}
for item in a['recovered_older_774_records'][1:]:
    row = package_lookup[item['path_in_774'][len(base):]]
    assert row['exported_sha256']==item['sha256']==item['package_exported_sha256']
    assert row['original_sha256']==item['package_original_sha256'] and row['bytes']==item['bytes']
old_locator = json.loads(older_records[base+'audit/continuation/rig/source-scope-reconciliation-044.json'])
loc = old_locator['new_variant_identity_and_range_map'][107]
assert loc['source_id'] == 'pr-working:crates/adventuresim-tactical-core/src/inventory/ordering.rs'
assert loc['recorded_sha256'] == worklist['roles'][1880]['historical_sha256']
older_list = a['recovered_older_774_records']
assert older_list[1]['package_original_sha256'] != older_list[1]['package_exported_sha256']
assert all(x['package_original_sha256']==x['package_exported_sha256'] for x in older_list[2:])
for kind, identity, index, field in [('camera','CAM-PPS-SERVER-PROJECTION-EQUIPMENT-055',55,'additional_actual_source_proofs'),('native','NATIVE-PS-SERVER-PLAYER-PROJECTION-026',26,'additional_actual_receivers')]:
    path = base+'audit/records/final-acceptance-20261021/wave-01/producer-supplement-after-owner/'+kind+'/traces.jsonl'
    line = json.loads(older_records[path].splitlines()[index-1])
    assert line.get('trace_id',line.get('id')) == identity
    source = line[field][4]
    assert source['source_id'] == loc['source_id'] and source['sha256'] == loc['recorded_sha256']
    assert source['read_ranges'] == [[1,130]] and source['actual_line_count'] == 130
q = load('RECONCILIATION-059.json')['entries']
for i in (1554,1555,1556,1557):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings = a['conditional_findings']
assert [x['id'] for x in findings] == ['REVIEW-DROPPED-SHIELD-CACHE-001','REVIEW-ITERATION-ARMOR-LAYER-LABELS-001']
assert all(x['accepted_historical_unique_count_change']==0 and x['repair']=='not started' and x['validation']=='not performed' for x in findings)
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert all(sum(x['id']==f['id'] and x['record']==name for x in ledger)==1 for f in findings)
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0
assert p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, separately attributed older receipts and bounded cache/report traces; no product execution.')
