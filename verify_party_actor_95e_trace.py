"""Verify bounded party admission evidence and its historical attribution.

This checks archived source bytes and records only; it does not run the product.
"""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-PARTY-ACTOR-95E.json'
doc = load(name)
ids = ['REVIEW-PARTY-ACTION-ACTOR-ADMISSION-001',
       'REVIEW-PARTY-LEADER-VOTE-ACTOR-ADMISSION-001']
assert doc['source_revision'] == revision and doc['lens'] == 3
assert [item['id'] for item in doc['candidates']] == ids

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == revision
    assert all(1 <= lo <= hi <= row['lines'] for lo, hi in row['ranges'])
    contents[row['path']] = raw

gov = contents['crates/adventuresim-stdb-module/src/strategic/governance.rs']
request = gov.split(b'pub fn request_party_action(', 1)[1].split(b'#[reducer]', 1)[0]
dismiss = gov.split(b'pub fn dismiss_party_action_request(', 1)[1].split(b'#[reducer]', 1)[0]
vote = gov.split(b'pub fn vote_for_party_leader(', 1)[1].split(b'\n}', 1)[0]
approval = gov.split(b'pub fn approve_party_action_request(', 1)[1].split(b'#[reducer]', 1)[0]
for section in (request, dismiss, vote):
    for guard in (b'require_strategic_gateway', b'require_strategic_character_authority', b'ctx.sender()'):
        assert guard not in section
assert b'require_living_character(ctx, requester_id)' in request
assert b'party.leader_id == requester_id' in request and b'allowed.contains(&action_kind.as_str())' in request
assert b'party_action_request_authority()' in request and b'.insert(PartyActionRequest' in request
assert b'require_living_character(ctx, leader_id)' in dismiss and b'party.leader_id != leader_id' in dismiss
assert b'.delete(request_id)' in dismiss
assert b'require_strategic_gateway(ctx)?' in approval and b'party.leader_id != leader_id' in approval
assert b'serde_json::from_str(&request.payload)' in approval and b'action.kind() != request.action_kind' in approval
for part in (b'if !voter.alive', b'require_no_unresolved_encounter(ctx, &party_id)?',
             b'!candidate.alive || candidate.party_id.as_deref() != Some(&party_id)',
             b'party_leader_vote().id().update(vote)', b'party_leader_vote().insert(vote)',
             b'normalize_and_elect_party_leader(ctx, &party_id)?'):
    assert part in vote
normalize = gov.split(b'pub(crate) fn normalize_and_elect_party_leader(', 1)[1].split(b'\n}', 1)[0]
assert b'leadership::elect_leader(' in normalize and b'party.leader_id = next;' in normalize
core = contents['crates/adventuresim-core/src/leadership.rs']
assert b'*count * 100 >= living.len() * 66' in core and 2 * 100 >= 3 * 66
character = contents['crates/adventuresim-stdb-module/src/character.rs']
living = character.split(b'pub fn require_living_character(', 1)[1].split(b'\n}', 1)[0]
assert b'.find(character_id)' in living and b'if !character.alive' in living
assert b'ctx.sender()' not in living and b'require_strategic' not in living
authority = contents['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
assert b'require_strategic_gateway(ctx).is_ok()' in authority
assert b'sender_owns_simulation_character(ctx, character_id)' in authority
simulation = contents['crates/adventuresim-stdb-module/src/simulation.rs']
assert b'run.owner == ctx.sender()' in simulation and b'.simulation_character()' in simulation
web = contents['crates/strategic-web/src/routes/parties.rs']
for fn_name, actor in ((b'update_party_check_targets', b'actor_id'),
                       (b'deny_action_request', b'leader_id'),
                       (b'vote_for_leader', b'voter_id')):
    part = web.split(b'async fn ' + fn_name, 1)[1].split(b'\n}', 1)[0]
    assert b'let Some(' + actor + b') = session.character_id_u64()' in part or \
           b'if let Some(' + actor + b') = session.character_id_u64()' in part
route = contents['crates/strategic-web/src/routes/mod.rs']
assert b'"request_party_action"' in route and b'json!(actor_id)' in route
for stem in ('request_party_action', 'dismiss_party_action_request', 'vote_for_party_leader'):
    client = contents[f'crates/adventuresim-stdb-client/src/{stem}_reducer.rs']
    assert stem.encode() in client and b'Reducer' in client

transport = {r['path']: r for r in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']
assert len(records) == 19
by = {}
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    entry = transport[row['original_path']]
    assert (row['sha256'], row['git_blob'], row['bytes']) == (entry['sha256'], entry['git_blob'], entry['bytes'])
    assert sha(raw) == row['published_sha256'] and len(raw) == row['published_bytes']
    if row['redacted_local_path_occurrences']:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    by[row['original_path']] = (row, json.loads(raw))

# References use original manifest hashes when publication required redaction.
def original_sha(path):
    return by[path][0]['sha256']

receipt = by['segovb/READ-L11-597.json'][1]
assert receipt['lens'] == 11 and receipt['source_revision'] == 'e2755a732c84e67296b5e3441a4259a84f461b2f'
assert receipt['accepted_unique_count_change'] == 0
assert receipt['other_lens_coverage_transferred'] is False
assert receipt['new_target_reconciliation_complete'] is False
assert receipt['implementation_gate_complete'] is False
assert original_sha(receipt['proposal']) == receipt['proposal_sha256']
assert original_sha(receipt['agent_verification']) == receipt['agent_verification_sha256']
for plan in receipt['plans']:
    assert original_sha(plan['path']) == plan['sha256']
proposal = by[receipt['proposal']][1]
assert proposal['lens'] == 11 and len(proposal['rows']) == len(receipt['rows']) == 4
for proposed, adopted in zip(proposal['rows'], receipt['rows']):
    assert proposed['registration'] == adopted['registration']
    assert adopted['attribution'].startswith(proposed['attribution'])
    assert 'root adopts without claiming personal reread' in adopted['attribution']
    assert proposed['assessed_ranges'] == adopted['assessed_ranges']
verify = by['segovb/VERIFY-L11-597.json'][1]
assert verify['receipt_sha256'] == original_sha('segovb/READ-L11-597.json')
assert verify['state_sha256'] == original_sha('segovb/STATE-L11-597.json')
assert verify['agent_attribution_preserved'] is True
for number, index in ((295, 1219), (296, 1220)):
    key = f'segovb/READ-L12-{number}.json'
    r = by[key][1]
    assert r['lens'] == 12 and [item['original_index'] for item in r['rows']] == [index]
    assert r['accepted_unique_count_change'] == 0 and r['current_source_reconciled'] is False
    assert r['implementation_gate_complete'] is False and original_sha(r['plan']) == r['plan_sha256']
    for item in r['rows']:
        assert original_sha(item['registration']) == item['registration_sha256']
        assert item['fresh_personal_body_reads'] == item['assessed_ranges']
        assert item['other_lens_verdict_transfer'] is False
    check = by[f'segovb/VERIFY-L12-{number}.json'][1]
    assert check['receipt_sha256'] == original_sha(key)
    assert check['state_sha256'] == original_sha(f'segovb/STATE-L12-{number}.json')
disposition = load('baseline-059/ledger.json')['segovb_continuation']['current_connected_dispositions'][132]
assert disposition['receipt']['sha256'] == original_sha('segovb/READ-L11-597.json')
assert disposition['scope_revision'] == receipt['source_revision']
assert disposition['latest_current_target_reconciled'] is False
assert disposition['disposition']['accepted_unique_count_change'] == 0

history_name = 'ASSESSMENT-L01-HISTORICAL-1219-1220.json'
history = load(history_name)['historical_source_roles']
assert [row['original_index'] for row in history] == [1219, 1220]
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == history_name and role['status'] == 'full-historical-body-trace-open'
        assert role['current_observations'] == row['prior_current_observations']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches = [entry for entry in ledger if entry['id'] == identity]
    assert len(matches) == 1 and matches[0]['record'] == name
progress = load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_admission_traces']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert progress['new_current_ledger_candidates'] == len(ledger)
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified party actor source evidence and historical attribution; no product ran')
