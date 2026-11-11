#!/usr/bin/env python3
"""Verify recorded consequence sources and original provenance; never run product code."""
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
predecessor = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
name = 'ASSESSMENT-COMBAT-CONSEQUENCE-RECEIPTS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2030-2032.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][pr]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == pr
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2030: 160, 2031: 166, 2032: 308}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed', 'bounded-current-trace-only')
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1, n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        verify_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
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
        record = load(prior['assessment'])
        assert any(x.get('path') == item['current_comparison']['path'] and x.get('revision') == prior['source_revision'] and x.get('ranges') == prior['ranges'] for x in walk(record)), (item['original_index'], prior)
        if prior['assessment'] == 'ASSESSMENT-L1-WHEEL-CUTOFF.json':
            assert prior['source_revision'] == predecessor and 'no independent95e credit' in item['prior_worklist_observation_revision_qualification']
        else: assert prior['source_revision'] == rev
condition, forecast, consequence = (current[i] for i in lengths)
viewer = sources['crates/adventuresim-tactical-core/src/player.rs']
receipt = sources['crates/adventuresim-tactical-server/src/mission/receipt.rs']
roster = sources['crates/adventuresim-core/src/mission/roster.rs']
constants = sources['crates/adventuresim-core/src/mission.rs']
module = sources['crates/adventuresim-stdb-module/src/tactical.rs']
consumer = sources['crates/adventuresim-stdb-module/src/strategic/autoresolve.rs']
ranged = sources['crates/adventuresim-tactical-server/src/combat/ranged.rs']
main = sources['crates/adventuresim-tactical-server/src/main.rs']
dev = sources['scripts/dev_stack.py']
assert b'let (limbs, skills, stats, attributes) = self.q_player.get(entity)?;' in viewer
assert b'let Ok(view) = viewer.get(entity)' in condition and b'let Ok(limbs) = limbs.get(entity)' in condition
assert b'+ state.fatigue' in condition and b'+ state.encumbrance' in condition
assert b'state.projected_increase.blood_loss' in forecast
assert consequence.count(b'defender_metadata.unwrap()') == 2 and consequence.count(b'attacker_metadata.unwrap()') == 1
assert b'defender_metadata.is_some_and(' in consequence and b'attacker_metadata.is_some_and(' in consequence
assert b'.min(adventuresim_core::mission::MAX_TACTICAL_AMMUNITION_USED)' in consequence
assert b'.find(|contact| contact.inventory_item_id == inventory_item_id)' in consequence
assert b'< adventuresim_core::mission::MAX_TACTICAL_EQUIPMENT_CONTACTS' in consequence
assert b'pub const MAX_TACTICAL_AMMUNITION_USED: u32 = 1_024;' in constants
assert b'pub const MAX_TACTICAL_EQUIPMENT_CONTACTS: usize = 128;' in constants
assert b'pub const MAX_TACTICAL_RECEIPT_PARTICIPANTS: usize = 16;' in constants
assert b'record_party_ammunition_use(consequences, *player_id)' in ranged
assert b'ammunition_used: consequence.ammunition_used' in receipt
assert b'party.truncate(adventuresim_core::mission::MAX_TACTICAL_RECEIPT_PARTICIPANTS)' in receipt
assert b'TacticalEquipmentContactRole::DefenderEquipment' in receipt and b'TacticalEquipmentContactRole::AttackerWeapon' in receipt
assert b'TacticalPartyRoster::try_from(authorized_party_member_ids)' in module
assert b'if !contacted_items.insert(contact.inventory_item_id)' in module
assert b'CatalogItemKind::Weapon | CatalogItemKind::Shield' in module
assert b'crate::repair::apply_impact(ctx, contact.inventory_item_id, contact.contact_stress)' in module
assert b'consequence.ammunition_used' in module and b'quantity.min(stack.quantity)' in consumer
assert b'if members.is_empty()' in roster and b'if members.len() > MAX_TACTICAL_RECEIPT_PARTICIPANTS' in roster
assert roster.count(b'.expect(') == 2
assert b'const MISSION_TIMEOUT_SECS: f32 = 300.0;' in main and b'(!args.no_timeout)' in main
assert b'"--no-timeout"' in dev
# Verify original059 records against the immutable transport and retain authors.
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery']
        assert body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else: assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob']
    records[item['original_path']] = body
path = 'segovb/READ-L12-681.json'; seg = json.loads(records[path]); check = json.loads(records['segovb/VERIFY-L12-681.json'])
assert [row['original_index'] for row in seg['rows']] == list(lengths)
assert all(row['fresh_personal_body_reads'] == [[1, lengths[row['original_index']]]] for row in seg['rows'])
assert seg['current_source_reconciled'] is False and seg['plan'] in records and seg['plan_sha256'] == lookup[seg['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['segovb/STATE-L12-681.json']['sha256']
for row in seg['rows']: assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
path = 'adler/READ-L11-315.json'; adler = json.loads(records[path]); check = json.loads(records['adler/VERIFY-L11-315.json'])
assert [row['original_index'] for row in adler['sources']] == [2029, 2030, 2031, 2032, 2033, 2034] and adler['current_reconciled'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-315.json']['sha256']
for row in adler['sources'] + adler.get('connected_reinspection', []):
    rp = 'adler/' + row['registration'].replace('\\', '/').split('/')[-1]
    assert rp in records and row['registration_sha256'] == lookup[rp]['sha256']
q = load('RECONCILIATION-059.json')['entries']; baseline_q = load('baseline-059/queue.json')['entries']
for i in (270, 1710, 1711, 1712, 1713, 1714):
    assert q[i]['contract_id'] == baseline_q[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    row = next(x for x in ledger if x['id'] == finding['id'])
    assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed'
assert a['conditional_findings'][0]['historical'] == 'baseline-059/ledger.json#/discovery_findings_and_dependencies/SEGOVB-L12-PARTY-METADATA-GUARDED-UNWRAP'
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical condition/consequence roles, predecessor qualifications and bounded terminal receipt sources; no product execution.')
