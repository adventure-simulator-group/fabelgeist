"""Verify selected threat-source evidence and historical attribution only."""
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
name = 'ASSESSMENT-L12-THREAT-MATERIALIZATION-95E.json'
doc = load(name)
ids = ['SEGOVB-L12-HOSTILE-THREAT-DROP-PRIVATE-READMISSION',
       'SEGOVB-L12-GENERATED-THREAT-PRIVATE-READMISSION']
assert doc['source_revision'] == rev and doc['lens'] == 12
assert [r['id'] for r in doc['candidates']] == ids
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
model = contents['crates/adventuresim-core/src/quest_generation/model.rs']
assert b'pub hostile_groups: Vec<(String, SiteId, ThreatId, u32)>' in model
bootstrap = contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
assert b'for (group_id, site_id, threat, count) in &generated.hostile_groups' in bootstrap
assert b'materialize_hostile_group(ctx, group_id, site, threat.as_str().into(), *count, 2)?' in bootstrap
authority = contents['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
body = authority.split(b'fn hostile_group_authority_row(',1)[1].split(b'/// Idempotency',1)[0]
assert b'enemy_type: String' in body and b'let threat = parse_threat(&enemy_type)?' in body
assert b'threat.profile().combat.escalation' in body and b'autoresolve_drop(&enemy_type)?' in body
assert body.index(b'let threat = parse_threat(&enemy_type)?') < body.index(b'autoresolve_drop(&enemy_type)?')
assert b'pub enemy_type: String' in authority
helper = contents['crates/adventuresim-stdb-module/src/strategic/autoresolve.rs']
assert b'fn autoresolve_drop(enemy_type: &str)' in helper and b'Ok(parse_threat(enemy_type)?.profile().combat.loot_item_id)' in helper
assert b'autoresolve_drop("bandit")?' in bootstrap
encounters = contents['crates/adventuresim-stdb-module/src/strategic/encounters.rs']
assert b'autoresolve_drop(&encounter.archetype)?' in encounters
bestiary = contents['crates/adventuresim-core/src/bestiary.rs']
assert b'pub fn try_new(value: &str)' in bestiary and b'impl FromStr for ThreatId' in bestiary
assert b'.monster(value)' in bestiary and b'pub fn profile(self) -> ThreatProfile' in bestiary
challenge = contents['crates/adventuresim-stdb-module/src/strategic/challenges.rs']
assert b'else if choice.requires_treatment()' in challenge and b'ordinary_treatment_complete' in challenge
assert b'for requirement in &selected.requirements' in challenge and b'for check in &selected.checks' in challenge
assert b'if ordinary_treatment && item_id == "bandage"' in challenge
assert b'if ordinary_treatment\n            && matches!(effect' in challenge
assert b'skills.religion_hours.direct(narrative_religion(*religion))' in challenge
assert b'crate::social::target_religion_check(ctx, character_id, religion)?' in challenge
catalog = contents['crates/adventuresim-core/src/road_encounter_catalog.rs']
assert b'pub enum ReligionId {\n    RomanCatholic,\n}' in catalog
knight = contents['content/encounters/wounded-knight-linden.yaml']
assert b'"skill": "physiology"' in knight and b'"item_id": "bandage"' in knight
web = contents['crates/strategic-web/src/templates/settlement/travel.rs']
assert b'disabled[!choice.available]' in web

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 38
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
for number, indices in ((283,[1203,1204]), (284,[1205]), (285,[1206]),
                        (286,[1207]), (287,[1208,1209]), (301,[1227])):
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
receipt = json.loads(by_path['segovb/READ-L12-301.json'])
parts = receipt['partial_sequence']
expected = [[[1,600]],[[601,1200]],[[1201,1800]],[[1801,2400]],[[2401,2973]]]
previous = None
for part, expected_range in zip(parts, expected, strict=True):
    raw = by_path[part['path']]; assert sha(raw) == part['sha256']
    fragment = json.loads(raw); assert fragment['fresh_personal_body_reads'] == expected_range
    assert fragment['other_lens_coverage_transferred'] is False and fragment['current_source_reconciled'] is False
    assert fragment['implementation_gate_complete'] is False
    if previous is not None:
        assert fragment['predecessor']['path'] == previous['path']
        assert fragment['predecessor']['sha256'] == previous['sha256']
    previous = part
assert json.loads(by_path[parts[0]['path']])['unread_tail'] == [[601,2973]]
assert json.loads(by_path[parts[3]['path']])['unread_tail'] == [[2401,2973]]
assert json.loads(by_path[parts[-1]['path']])['cumulative_personal_body_reads'] == [[1,2973]]
assert json.loads(by_path[parts[-1]['path']])['unread_tail'] == []

histname = 'ASSESSMENT-L01-HISTORICAL-1203-1208.json'
history = load(histname)['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1203, 1209))
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
        assert role.get('current_observations',[]) == row['prior_current_observations']
q = load('RECONCILIATION-059.json')['entries']
for index, contract in [(228,ids[0]), (241,ids[1]),
                        (974,'Q-L12-f917f8cdb3a3f2044f43'),
                        (975,'Q-L12-d3c2d0c28c4cc6667828'),
                        (977,'Q-L12-b29e5642ee16750db644')]:
    assert q[index]['contract_id'] == contract and q[index]['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches = [row for row in ledger if row['id'] == identity]
    assert len(matches) == 1 and matches[0]['record'] == name
progress = load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_string_traces']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert progress['new_current_ledger_candidates'] == len(ledger)
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified threat materialization trace and earlier fragments; no product ran')
