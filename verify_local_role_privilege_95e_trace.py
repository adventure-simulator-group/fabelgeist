"""Verify preserved source and earlier role evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L12-LOCAL-ROLE-PRIVILEGE-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'REVIEW-LOCAL-ROLE-EQUIPMENT-001'
assert doc['source_revision'] == rev and doc['lens'] == 12
assert doc['candidates'][0]['id'] == cid

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines'] and row['revision'] == rev
    assert all(1 <= lo <= hi <= row['lines'] for lo, hi in row['ranges'])
    contents[row['path']] = raw
core = contents['crates/adventuresim-core/src/organization.rs']
assert b'pub privileges: Vec<Privilege>' in core
assert b'Organization privileges are inherited by every role; role privileges' in core
assert b'self.has_privilege(privilege)' in core and b'role.privileges.contains(&privilege)' in core
validator = contents['crates/adventuresim-core/src/organization_catalog_validation.rs']
assert b'array(role, source, "privileges")?' in validator
assert b'array(org, source, "privileges")?' in validator
assert b'"bear_arms"' in validator and b'"wear_armor"' in validator
build = contents['crates/adventuresim-core/build.rs']
assert b'root.join("content/organizations")' in build and b'fs::read_dir(&content)' in build
assert b'organization_catalog_validation::validate_documents(' in build
local = contents['crates/adventuresim-stdb-module/src/organization.rs']
local_fn = local.split(b'pub fn presented_privilege(', 1)[1].split(b'\n}', 1)[0]
assert b'definition.recognition.includes(settlement_id)' in local_fn
assert b'definition.has_privilege(privilege)' in local_fn and b'active_membership(' in local_fn
assert b'membership_role(' not in local_fn and b'has_privilege_at_role(' not in local_fn
other_fn = local.split(b'pub fn global_presented_privilege(', 1)[1].split(b'\n}', 1)[0]
assert b'membership_role(' in other_fn and b'current_membership_grants(' in other_fn
membership = contents['crates/adventuresim-stdb-module/src/organization/membership.rs']
assert b'membership_is_current(membership, minute)' in membership
assert b'definition.has_privilege_at_role(&role.id, privilege)' in membership
law = contents['crates/adventuresim-stdb-module/src/equipment_law.rs']
assert b'Privilege::BearArms' in law and b'Privilege::WearArmor' in law
assert b'crate::organization::presented_privilege(' in law
assert b'if require_item_legal(ctx, character_id, inventory_item_id).is_err()' in law
assert b'unequip_wearable(ctx, inventory_item_id)' in law
character = contents['crates/adventuresim-stdb-module/src/character.rs']
assert b'crate::equipment_law::require_item_legal(ctx, character_id, inventory_item_id)?' in character
forage = contents['crates/adventuresim-stdb-module/src/foraging.rs']
assert forage.count(b'crate::organization::global_presented_privilege(') >= 2

# Recompute the exact tracked authored inventory, using stored bytes and tree identities.
inv = doc['authoring_inventory']
paths = sorted(p for p in tree if p.startswith('content/organizations/')
               and '/' not in p[len('content/organizations/'):]
               and p.endswith('.yaml'))
assert paths == [x['path'] for x in inv['files']]
for item in inv['files']:
    orgs = json.loads(contents[item['path']])
    expected = [{'id': o['id'], 'privileges': o.get('privileges', []),
                 'roles': [{'id': r['id'], 'privileges': r.get('privileges', [])}
                           for r in o['roles']]} for o in orgs]
    assert item['organizations'] == expected
    assert item['organization_count'] == len(orgs)
    assert item['role_count'] == sum(len(o['roles']) for o in orgs)
assert inv['total_organizations'] == sum(x['organization_count'] for x in inv['files']) == 25
assert inv['total_roles'] == sum(x['role_count'] for x in inv['files']) == 59
gear = [{'organization': o['id'], 'role': r['id'],
         'privileges': [p for p in r['privileges'] if p in ('bear_arms', 'wear_armor')]}
        for f in inv['files'] for o in f['organizations'] for r in o['roles']
        if any(p in ('bear_arms', 'wear_armor') for p in r['privileges'])]
assert gear == inv['role_entries_with_bear_arms_or_wear_armor'] == []
assert inv['tracked_settlement_policies'] == json.loads(contents['content/settlement-policies.yaml'])

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']
assert len(records) == 7
by_path = {}
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    original = transport[row['original_path']]
    assert row['sha256'] == original['sha256'] and row['git_blob'] == original['git_blob']
    assert row['bytes'] == original['bytes'] and sha(raw) == row['published_sha256']
    assert len(raw) == row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    by_path[row['original_path']] = raw
receipt = json.loads(by_path['segovb/READ-L12-262.json'])
assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == [1145, 1146, 1147]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['implementation_gate_complete'] is False
assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
for row in receipt['rows']:
    assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
verification = json.loads(by_path['segovb/VERIFY-L12-262.json'])
assert verification['receipt_sha256'] == sha(by_path['segovb/READ-L12-262.json'])
assert verification['state_sha256'] == sha(by_path['segovb/STATE-L12-262.json'])

history = load('ASSESSMENT-L01-HISTORICAL-1141-1147.json')['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1141, 1148))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1141-1147.json'
        assert role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1145:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
q = load('RECONCILIATION-059.json')['entries'][952]
assert q['contract_id'] == 'Q-L12-71b394d526b7c691077b'
assert q['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L12-LOCAL-ROLE-PRIVILEGE-95E.json']) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified local privilege source, tracked content inventory and earlier attribution; no product ran')
