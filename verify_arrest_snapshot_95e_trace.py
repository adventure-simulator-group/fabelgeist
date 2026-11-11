"""Verify the selected arrest guard and earlier evidence without product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L01-ARREST-SNAPSHOT-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
assert doc['source_revision'] == rev and doc['lens'] == 1 and doc['candidates'] == []

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
reputation = contents['crates/adventuresim-stdb-module/src/reputation.rs']
snapshot = reputation.split(b'pub fn snapshot_arrest_charges(', 1)[1].split(b'\n}', 1)[0]
assert b'unsettled_local_offenses(ctx, character_id, settlement_id)' in snapshot
assert b'format!("{incident_id}:{}", offense.id)' in snapshot
assert b'authority_arrest_charge().id().find(&id).is_none()' in snapshot
assert b'authority_arrest_charge()' in snapshot and b'.insert(AuthorityArrestCharge' in snapshot
local = reputation.split(b'pub fn unsettled_local_offenses(', 1)[1].split(b'\n}', 1)[0]
assert b'!offense.settled' in local and b'offense.settlement_id == settlement_id' in local
assert b'(left.occurred_at_minute, &left.id)' in local
incidents = contents['crates/adventuresim-stdb-module/src/strategic/incidents.rs']
creation = incidents.split(b'fn create_strategic_incident(', 1)[1].split(b'\n}', 1)[0]
pending = creation.index(b'.any(|incident| incident.status == IncidentStatus::Pending)')
assert pending < creation.index(b'.find(|incident| incident.source_id == source_id)')
assert b'return Ok(None);' in creation[pending:creation.index(b'.find(|incident| incident.source_id == source_id)')]
caller = incidents.split(b'pub(crate) fn maybe_trigger_activity_incident(', 1)[1].split(b'\n}', 1)[0]
assert b'authority_arrest_source_id(' in caller
assert b'if let Some(incident_id) = incident.as_ref()' in caller
assert b'incident_kind == IncidentKind::AuthorityArrest' in caller
assert b'snapshot_arrest_charges(' in caller
fingerprint = incidents.split(b'fn authority_arrest_source_id(', 1)[1].split(b'\n}', 1)[0]
assert b'for offense in offenses' in fingerprint and b'hasher.update(offense.id.as_bytes())' in fingerprint
assert b'&fingerprint[..16]' in fingerprint
surrender = incidents.split(b'pub fn surrender_to_authority(', 1)[1].split(b'\n}', 1)[0]
assert b'unsettled_arrest_charges(' in surrender and b'settle_offenses(ctx, offenses)' in surrender
fixture = contents['crates/adventuresim-stdb-module/src/world_event/authority_tests.rs']
assert b'snapshot_arrest_charges(ctx, "incident:authority-offense"' in fixture
assert b'authority_test_offense_policy' in fixture

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 24
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
for number, indices in ((271, list(range(1173, 1177))), (272, [1177, 1178]), (273, [1179])):
    name = f'segovb/READ-L12-{number}.json'; receipt = json.loads(by_path[name])
    assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
    for row in receipt['rows']:
        assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
    check = json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256'] == sha(by_path[name])
    assert check['state_sha256'] == sha(by_path[f'segovb/STATE-L12-{number}.json'])
last = json.loads(by_path['segovb/READ-L12-273.json'])
assert last['rows'][0]['assessed_ranges'] == [[74, 179], [258, 361]]
locator = last['rows'][0]['function_contract']['source_locator_qualification']
assert locator['historical_locator_proof_body_not_read'] is True
assert sha(by_path[locator['path']]) == locator['sha256']
assert len(last['connected_extensions']) == 2
for extension in last['connected_extensions']:
    assert sha(by_path[extension['registration']]) == extension['registration_sha256']
    assert sha(by_path[extension['receipt']]) == extension['receipt_sha256']
assert last['connected_extensions'][0]['fresh_personal_body_reads'] == [[1, 73], [180, 189], [246, 257]]
assert last['connected_extensions'][1]['fresh_personal_body_reads'] == [[190, 245]]

history = load('ASSESSMENT-L01-HISTORICAL-1173-1179.json')['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1173, 1180))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1173-1179.json'
        assert role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1177:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
q = load('RECONCILIATION-059.json')['entries'][964]
assert q['contract_id'] == 'Q-L12-e302315e0fe26cd42392' and q['current_reconciliation'].startswith('partial ')
progress = load('CONTINUATION-PROGRESS-95E.json')
assert 'ASSESSMENT-L01-ARREST-SNAPSHOT-95E.json' in progress['new_nonfinding_assessments']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected arrest guard and preserved partial/extension attribution; no product ran')
