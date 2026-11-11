"""Verify wedding scheduling source and earlier evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L07-WEDDING-FRONTIER-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'REVIEW-WEDDING-GLOBAL-FRONTIER-GATE-001'
assert doc['source_revision'] == rev and doc['lens'] == 7 and doc['candidates'][0]['id'] == cid

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
causal = contents['crates/adventuresim-stdb-module/src/npc_causal.rs']
tick = causal.split(b'pub fn run_npc_causal_tick(', 1)[1].split(b'\n}', 1)[0]
assert tick.index(b'settle_due_lifecycle_events_global(') < tick.index(b'stable_npc_cohort(')
assert b'advance_stationary_character_to(' in tick
queue = contents['crates/adventuresim-stdb-module/src/relationship/mod.rs']
combined = queue.split(b'pub fn settle_due_lifecycle_events_global(', 1)[1].split(b'\n}', 1)[0]
assert b'row.effective_minute <= now' in combined
assert combined.index(b'due.retain(|event| event.processable(ctx))') < combined.index(b'due.truncate(limit)')
assert b'settle_due_weddings(ctx, participant_id, effective_minute)' in combined
lifecycle = contents['crates/adventuresim-stdb-module/src/relationship/lifecycle.rs']
processable = lifecycle.split(b'fn processable(&self', 1)[1].split(b'\nfn record_lifecycle_failure(', 1)[0]
assert b'participant_died_before_ceremony' in processable
assert b'death.strategic_minute <= *effective_minute' in processable
assert b'participants.into_iter().all(|character_id|' in processable
assert b'canonical_now(ctx, character_id)' in processable and b'frontier >= *effective_minute' in processable
assert b'Self::Birth {' in processable and b'canonical_now(ctx, *mother_id)' in processable
marriage = contents['crates/adventuresim-stdb-module/src/relationship/marriage.rs']
wedding = marriage.split(b'pub fn settle_due_weddings(', 1)[1].split(b'pub fn settle_due_weddings_global(', 1)[0]
assert b'_participant_frontier: StrategicMinute' in wedding
assert b'let now = crate::time::refresh_clock(ctx)?' in wedding
assert b'row.effective_minute <= now' in wedding
assert b'Subjective character clocks\n        // neither delay the ceremony' in wedding
separate_global = marriage.split(b'pub fn settle_due_weddings_global(', 1)[1].split(b'\n}', 1)[0]
assert b'settle_due_weddings(ctx, commitment.first_character_id, now)?' in separate_global
personal = contents['crates/adventuresim-stdb-module/src/time/mod.rs']
assert b'settle_due_weddings(ctx, character_id, minute)?' in personal
wedding_test = contents['crates/adventuresim-stdb-module/src/relationship/tests/chronology.rs']
assert b'exclusive_scope_and_weddings_use_canonical_world_time' in wedding_test
assert b'!wedding.contains("all_participants_reached_ceremony")' in wedding_test
other_test = contents['crates/adventuresim-stdb-module/src/relationship/tests/lifecycle.rs']
assert b'birth_and_discovery_wait_for_authoritative_personal_frontiers' in other_test
assert b'dead_fiance_is_processable_without_reaching_the_ceremony' in other_test

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 7
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
name = 'segovb/READ-L12-268.json'; receipt = json.loads(by_path[name])
assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == [1160, 1161, 1162]
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['implementation_gate_complete'] is False
assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
for row in receipt['rows']:
    assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
check = json.loads(by_path['segovb/VERIFY-L12-268.json'])
assert check['receipt_sha256'] == sha(by_path[name])
assert check['state_sha256'] == sha(by_path['segovb/STATE-L12-268.json'])

history = load('ASSESSMENT-L01-HISTORICAL-1160-1162.json')['historical_source_roles']
assert [r['original_index'] for r in history] == [1160, 1161, 1162]
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1160-1162.json'
        assert role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1162:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
q = load('RECONCILIATION-059.json')['entries'][957]
assert q['contract_id'] == 'Q-L12-7ebcf2f8f040e4691baf' and q['current_reconciliation'].startswith('partial ')
base = load('BASELINE-95E-20261110.json')
ledger = base['newly_assessed_ledger']['entries']
assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L07-WEDDING-FRONTIER-95E.json']) == 1
assert load(base['latest_main_recheck'])['observed_revision'] == rev
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified wedding selectors, current revision observation and prior attribution; no product ran')
