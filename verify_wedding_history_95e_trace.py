"""Verify historical wedding projection evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L13-WEDDING-HISTORY-PROJECTION-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'PL13-281-WEDDING-HISTORICAL-PROJECTION-PENDING'
assert doc['source_revision'] == rev and doc['lens'] == 13 and doc['candidates'][0]['id'] == cid

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
projection = contents['crates/adventuresim-stdb-module/src/relationship/projections.rs']
view = projection.split(b'pub fn backend_character_relationship_statuses(', 1)[1].split(b'\n}', 1)[0]
assert b'if !is_strategic_gateway(ctx)' in view
assert b'let observer_minute' in view and b'.character_time()' in view
assert b'courtship.first_character_id' in view and b'courtship.second_character_id' in view
assert b'marriage.married_minute <= observer_minute' in view
wedding = view.split(b'let wedding = ctx', 1)[1].split(b'BackendCharacterRelationshipStatus {', 1)[0]
assert b'.exclusive_commitment_participant()' in wedding
assert b'.find(character.id)' in wedding and b'.find(&participant.commitment_id)' in wedding
assert b'commitment.created_minute <= observer_minute' in wedding
assert b'.is_none_or(|resolved| resolved > observer_minute)' in wedding
assert b'wedding_effective_minute: wedding.as_ref().map' in view
commitments = contents['crates/adventuresim-stdb-module/src/relationship/commitments.rs']
terminal = commitments.split(b'fn transition_commitment_terminal(', 1)[1].split(b'\n}', 1)[0]
assert b'.exclusive_commitment_participant()' in terminal and b'.delete(character_id)' in terminal
assert b'commitment.resolved_minute = Some(minute)' in terminal
assert b'.exclusive_commitment()' in terminal and b'.update(commitment.clone())' in terminal
marriage = contents['crates/adventuresim-stdb-module/src/relationship/marriage.rs']
assert b'_participant_frontier: StrategicMinute' in marriage
assert b'let now = crate::time::refresh_clock(ctx)?' in marriage
assert b'CommitmentTerminalReason::WeddingCompleted' in marriage
assert b'settle_due_weddings(ctx, character_id, minute)?' in contents['crates/adventuresim-stdb-module/src/time/mod.rs']
web = contents['crates/strategic-web/src/routes/dialogue.rs']
assert b'row.wedding_commitment_id.is_some()' in web and b'row.wedding_partner_id == Some(target_id)' in web
assert b'calendar_countdown_days(actor_minute, commitment.wedding_effective_minute.as_ref())' in web
assert b'romantic_actions.push(RomanceAction::ScheduleWedding)' in web
residence = contents['crates/strategic-web/src/routes/settlements/residences.rs']
assert b'character_relationship_status_by_character_id(character.id)' in residence
assert b'.wedding_effective_minute' in residence and b'.map(|date| WeddingPresentation {' in residence
party = contents['crates/strategic-web/src/routes/settlements/party/social.rs']
assert b'status.wedding_commitment_id.is_some()' in party
# Illustrative observer ordering only; no product functions, databases, or fixtures are executed.
assert 10 <= 50 < 100

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 28
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
for number, indices in ((269, [1163, 1164, 1165]), (270, list(range(1166, 1173)))):
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
adopt = json.loads(by_path['segovb/READ-L13-PARALLEL-279.json'])
assert adopt['lens'] == 13 and adopt['root_personal_rereads'] == []
assert adopt['accepted_unique_count_change'] == 0 and adopt['current_source_reconciled'] is False
assert adopt['implementation_gate_complete'] is False
trim = lambda path: path.removeprefix('target/726-audit-adler-20261027/')
for key in ('delegated_receipt', 'delegated_verification', 'agent_validation_manifest'):
    link = adopt[key]; assert sha(by_path[trim(link['path'])]) == link['sha256']
prior = json.loads(by_path['parallel-l13-20261029/READ-PL13-281.json'])
assert prior['lens'] == 13 and prior['accepted_unique_count_change'] == 0
assert prior['partial_read_union'][0]['original'] == 1162
assert prior['partial_read_union'][0]['earlier_personal_ranges'] == [[1, 250]]
assert prior['partial_read_union'][0]['fresh_completion_ranges'] == [[251, 496]]
partial = prior['partial_checkpoints'][0]
assert sha(by_path[trim(partial['path'])]) == partial['sha256']
assert any(row['original'] == 1164 for row in prior['rows'])
root_check = json.loads(by_path['segovb/VERIFY-L13-PARALLEL-279.json'])
assert root_check['receipt_sha256'] == sha(by_path['segovb/READ-L13-PARALLEL-279.json'])
assert root_check['state_sha256'] == sha(by_path['segovb/STATE-L13-PARALLEL-279.json'])

history = load('ASSESSMENT-L01-HISTORICAL-1163-1172.json')['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1163, 1173))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1163-1172.json'
        assert role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1165:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
queue = load('RECONCILIATION-059.json')['entries']
for i, name in ((959, 'Q-L12-676b0782af1fde8ff508'), (2469, cid)):
    assert queue[i]['contract_id'] == name and queue[i]['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L13-WEDDING-HISTORY-PROJECTION-95E.json']) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified wedding history source, independent attributions and archived roles; no product ran')
