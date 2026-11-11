"""Verify social trace and original fragment attribution without product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L12-SOCIAL-ACTION-95E.json'
doc = load(name)
assert doc['source_revision'] == rev and doc['lens'] == 12
assert [r['id'] for r in doc['candidates']] == ['SEGOVB-L12-SOCIAL-ACTION-PRIVATE-READMISSION']
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == rev and all(1 <= lo <= hi <= row['lines'] for lo, hi in row['ranges'])
    contents[row['path']] = raw
social = contents['crates/adventuresim-stdb-module/src/social.rs']
auto = social.split(b'pub(crate) fn apply_automatic_social_chats(',1)[1].split(b'pub(crate) fn prune_social_addresses',1)[0]
assert b'automatic_social_action(ctx, actor_id, target_id, topic)?' in auto
assert b'action.reducer_value().into()' in auto and b'perform_social_action_authoritative(' in auto
shared = social.split(b'fn perform_social_action_authoritative(',1)[1].split(b'pub(crate) fn apply_automatic_social_chats',1)[0]
assert b'action_kind: String' in shared and b'admission::admit_social_action(ctx, actor_id, target_id, &action_kind)?' in shared
assert b'action.skill(' in shared and b'action.available_for(topic)' in shared
manual = social.split(b'pub fn perform_social_action(',1)[1].split(b'fn known_belief_for_axis',1)[0]
assert b'require_strategic_gateway(ctx)?' in manual and b'perform_social_action_authoritative(' in manual
admission = contents['crates/adventuresim-stdb-module/src/social/admission.rs']
assert b'.parse::<SocialActionKind>()' in admission and b'validate_social_pair(ctx, &actor, &target, is_self)?' in admission
core = contents['crates/adventuresim-core/src/social/action_kind.rs']
assert b'Self::Rally => "command"' in core and b'Self::Reframe => "deception"' in core
assert b'#[serde(rename = "command")]' in core and b'#[serde(rename = "deception")]' in core
assert b'impl FromStr for SocialActionKind' in core
web = contents['crates/strategic-web/src/routes/settlements/party/social.rs']
assert b'let Some(actor_id) = session.character_id_u64()' in web and b'json!(form.action_kind)' in web
witness = contents['crates/adventuresim-core/src/social/witness.rs']
assert b'let succeeded = !input.claim_is_factually_accurate && outcome.succeeded;' in witness
assert b'outcome.succeeded && capability.has_bound_concern && !capability.bound_released' in social
helper = social.split(b'pub(crate) fn apply_corpse_family_offense(',1)[1].split(b'\n}',1)[0]
assert b'put_affinity(' in helper and b'record_morale_event(' in helper and b'event_id' in helper
condition = contents['crates/adventuresim-stdb-module/src/condition.rs']
record = condition.split(b'pub fn record_morale_event(',1)[1].split(b'\n}',1)[0]
assert b'ctx.db.morale_event().insert(MoraleEvent' in record and b'if magnitude == 0.0' in record
corpse = contents['crates/adventuresim-stdb-module/src/corpse.rs']
permission = corpse.split(b'pub(crate) fn grant_permission_from_dialogue(',1)[1].split(b'\nfn permission_scope_label',1)[0]
assert b'if let Some(attempt)' in permission and b'return Ok(attempt.granted);' in permission
assert permission.index(b'return Ok(attempt.granted);') < permission.index(b'apply_corpse_family_offense(')
assert b'family-bypassed:{corpse_id}:{}' in permission
for method, following in [(b'pub fn examine_corpse(',b'pub fn open_corpse('),
                          (b'pub fn open_corpse(',b'pub fn exhume_corpse('),
                          (b'pub fn exhume_corpse(',b'pub fn bury_corpse('),
                          (b'pub fn burn_corpse(',b'pub(crate) fn grant_permission_from_dialogue(')]:
    body = corpse.split(method,1)[1].split(following,1)[0]
    assert b'.autopsy_action_receipt()' in body and b'return Ok(());' in body
    needle = b'apply_corpse_family_offense(' if method == b'pub fn burn_corpse(' else b'apply_unauthorized_consequences('
    assert body.index(b'.autopsy_action_receipt()') < body.index(needle)
assert b'row.scope == required_scope || row.scope == CorpsePermissionScope::Exhumation' in corpse

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 26
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
for number, indices in ((280,[1195]), (281,[1196,1197,1198,1199,1200,1201]), (282,[1202])):
    receipt_name = f'segovb/READ-L12-{number}.json'; receipt = json.loads(by_path[receipt_name])
    assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
    for row in receipt['rows']:
        assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
        assert row['fresh_personal_body_reads'] == row['assessed_ranges']
        assert row['other_lens_verdict_transfer'] is False
    check = json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256'] == sha(by_path[receipt_name])
    assert check['state_sha256'] == sha(by_path[f'segovb/STATE-L12-{number}.json'])
first = json.loads(by_path['segovb/READ-L12-280.json'])['rows'][0]
parts = first['function_contract']['read_attribution']['parts']
assert [part['fresh'] for part in parts] == [[[1,600]],[[601,1200]],[[1201,1800]],[[1801,2400]],[[2401,3000]],[[3001,3800]]]
for part in parts:
    assert sha(by_path[part['path']]) == part['sha256']
    fragment = json.loads(by_path[part['path']])
    assert fragment['fresh_personal_body_reads'] == part['fresh']
    assert fragment['other_lens_coverage_transferred'] is False and fragment['current_source_reconciled'] is False
    assert fragment['implementation_gate_complete'] is False
assert json.loads(by_path[parts[0]['path']])['unread_tail'] == [[601,3800]]
assert json.loads(by_path[parts[-1]['path']])['cumulative_personal_body_reads'] == [[1,3800]]

histname = 'ASSESSMENT-L01-HISTORICAL-1195-1202.json'
history = load(histname)['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1195, 1203))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == histname and role['status'] == 'full-historical-body-trace-open'
q = load('RECONCILIATION-059.json')['entries']
for index, contract in [(227,'SEGOVB-L12-SOCIAL-ACTION-PRIVATE-READMISSION'),
                        (971,'Q-L12-31ad546cf6c0b655d59d'),
                        (972,'Q-L12-385e94497aa5f2b759ee'),
                        (973,'Q-L12-39c4f2d4e1d138a59255')]:
    assert q[index]['contract_id'] == contract and q[index]['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
matches = [row for row in ledger if row['id'] == doc['candidates'][0]['id']]
assert len(matches) == 1 and matches[0]['record'] == name
progress = load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_string_traces']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert progress['new_current_ledger_candidates'] == len(ledger)
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified social action trace, bounded guards and earlier fragments; no product ran')
