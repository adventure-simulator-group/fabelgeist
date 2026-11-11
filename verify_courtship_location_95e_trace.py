"""Verify courtship source and historical evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L05-COURTSHIP-LOCATION-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'REVIEW-COURTSHIP-ABSENT-LOCATION-001'
assert doc['source_revision'] == rev and doc['lens'] == 5 and doc['candidates'][0]['id'] == cid

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
courtship = contents['crates/adventuresim-stdb-module/src/relationship/courtship.rs']
pair = courtship.split(b'fn validate_canonical_courtship_pair(', 1)[1].split(b'\nfn establish_courtship(', 1)[0]
assert b'if suitor.current_settlement_id != partner.current_settlement_id' in pair
assert b'CourtshipRejectionCode::CoLocation' in pair
assert b'character_case_site' not in pair and b'current_settlement_id.is_none()' not in pair
assert b'TemporalScope::ExclusiveShared' in pair
npc = courtship.split(b'pub(crate) fn establish_npc_courtship_and_wedding(', 1)[1].split(b'\n#[reducer]', 1)[0]
assert b'suitor.current_settlement_id.is_none()' in npc
assert b'suitor.current_settlement_id != partner.current_settlement_id' in npc
begin = courtship.split(b'pub fn begin_informal_courtship(', 1)[1].split(b'\n}', 1)[0]
assert b'require_strategic_character_authority(ctx, suitor_id)' in begin
assert b'validate_canonical_courtship_pair(ctx, suitor_id, partner_id)' in begin
assert b'affinity_at(ctx, partner_id, suitor_id, minute)' in begin
assert b'establish_courtship(' in begin
establish = courtship.split(b'fn establish_courtship(', 1)[1].split(b'\n#[derive', 1)[0]
assert b'let pair_settlement' in establish
assert b'observer.current_settlement_id != pair_settlement' in establish
assert b'courtship_observer_baseline()' in establish
relationship_model = contents['crates/adventuresim-stdb-module/src/relationship/model.rs']
assert b'Observer eligibility and skill are frozen when the relationship starts.' in relationship_model
exclusive = relationship_model.split(b'TemporalScope::ExclusiveShared => {', 1)[1].split(b'\n        }', 1)[0]
assert b'canonical_now(ctx, actor_id)' not in exclusive  # actor clock is read before the match
assert b'crate::time::refresh_clock(ctx)' in exclusive and b'current_settlement_id' not in exclusive
assert b'let actor_minute = canonical_now(ctx, actor_id)?' in relationship_model
observer = contents['crates/adventuresim-stdb-module/src/relationship/courtship_discovery.rs']
assert b'.courtship_observer_baseline()' in observer and b'character_alive_at(ctx, baseline.observer_id' in observer
assert b'current_settlement_id' not in observer
source_test = contents['crates/adventuresim-stdb-module/src/relationship/tests/courtship_discovery.rs']
assert b'fn discovery_attempts_use_frozen_observers_and_weaker_deception()' in source_test
assert b'courtship_observer_baseline()' in source_test
travel = contents['crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs']
assert b'member.current_settlement_id = None;' in travel
assert b'crate::investigation::set_character_case_site(' in travel
place = contents['crates/adventuresim-stdb-module/src/investigation/model.rs']
assert b'pub struct CharacterCaseSiteOccupancy' in place and b'pub case_site_id: CaseSiteId' in place
security = contents['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
assert b'require_strategic_gateway(ctx).is_ok()' in security
assert b'sender_owns_simulation_character(ctx, character_id)' in security
web = contents['crates/strategic-web/src/routes/dialogue.rs']
assert b'character.current_settlement_id.as_deref() != Some(settlement_id)' in web
assert b'presence.settlement_id == settlement_id' in web and b'presence.location_id == location_id' in web
assert b'let npc = available_social_npc(' in web
binding = contents['crates/adventuresim-stdb-client/src/begin_informal_courtship_reducer.rs']
assert b'pub suitor_id: u64' in binding and b'pub partner_id: u64' in binding

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 19
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
for number, indices in ((265, [1153, 1154]), (266, [1155, 1156]), (267, [1157, 1158, 1159])):
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

history = load('ASSESSMENT-L01-HISTORICAL-1152-1159.json')['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1152, 1160))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1152-1159.json'
        assert role['status'] == 'full-historical-body-trace-open'
q = load('RECONCILIATION-059.json')['entries'][956]
assert q['contract_id'] == 'Q-L12-0bb3ceca8f6cbe83fca5' and q['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L05-COURTSHIP-LOCATION-95E.json']) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified courtship location, frozen cohort and historical attribution; no product ran')
