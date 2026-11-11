"""Verify pinned outbreak source and earlier evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L12-OUTBREAK-SITE-EXPOSURE-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
ids = ['REVIEW-OUTBREAK-SITE-SOURCE-SELECTION-001',
       'SEGOVB-L12-OUTBREAK-SOURCE-KIND-RAW-DISPATCH']
assert doc['source_revision'] == rev and doc['lens'] == 12
assert [c['id'] for c in doc['candidates']] == ids

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
model = contents['crates/adventuresim-core/src/quest_generation/model.rs']
source = model.split(b'pub enum OutbreakSource {', 1)[1].split(b'\n}', 1)[0]
for variant in (b'Sanitation {', b'Behavior {', b'ThreatVector {', b'Environmental {'):
    assert variant in source
assert b'#[serde(tag = "kind", rename_all = "snake_case")]' in model
assembly = contents['crates/adventuresim-core/src/quest_generation/assembly.rs']
assert b'DiseaseId::ShroudFever => (' in assembly and b'OutbreakSource::Environmental {' in assembly
assert b'DiseaseId::Mahrdruck => (' in assembly and b'OutbreakSource::ThreatVector {' in assembly
assert b'disease_intensity: 360,' in assembly
assert b'kind: InvestigationActionKind::InspectSite' in assembly
assert b'target_id: source_site.0.clone()' in assembly
problem = contents['crates/adventuresim-stdb-module/src/local_problem.rs']
material = problem.split(b'pub(crate) fn materialize_generated_problem(', 1)[1].split(b'ctx.db.local_problem_symptom()', 1)[0]
assert b'settlement_id: settlement_id.into()' in material
assert b'id: case.problem_id.clone()' in material
assert b'disease_intensity: consequence.effects.disease_intensity' in material
assert b'|outbreak| outbreak.disease.stable_id().into()' in material
bootstrap = contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
assert b'crate::local_problem::materialize_generated_problem(ctx, generated, settlement_id)?' in bootstrap
assert b'crate::outbreak::materialize_generated_outbreak(' in bootstrap
outbreak = contents['crates/adventuresim-stdb-module/src/outbreak.rs']
assert b'pub source_kind: String' in outbreak and b'pub source_json: String' in outbreak
assert b'let (source_kind, carrier) = match &outbreak.source' in outbreak
assert b'serde_json::to_string(&outbreak.source)' in outbreak
assert b'authority.source_kind == "threat_vector"' in outbreak
assert b'authority.source_kind != "threat_vector"' in outbreak
window = outbreak.split(b'pub(crate) fn exposure_windows(', 1)[1].split(b'pub(crate) fn record_case_site_presence_transition(', 1)[0]
assert b'match authority.source_kind.as_str()' in window
assert b'"environmental" | "threat_vector"' in window
assert b'.outbreak_source_presence_span()' in window and b'span.source_place_id' in window
assert b'to.min(authority.remediated_at.unwrap_or(to))' in window
transition = outbreak.split(b'pub(crate) fn record_case_site_presence_transition(', 1)[1].split(b'pub(crate) fn discover_case_corpses(', 1)[0]
assert b'authority.remediated_at.is_none()' in transition and b'fixture.place() == &destination_place' in transition
assert b'.outbreak_source_presence_span()' in transition
occupancy = contents['crates/adventuresim-stdb-module/src/investigation/model.rs']
assert b'crate::outbreak::record_case_site_presence_transition(' in occupancy
travel = contents['crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs']
arrival = travel.split(b'fn travel_to_case_site_impl(', 1)[1].split(b'fn ', 1)[0]
assert b'member.current_settlement_id = None;' in arrival
assert b'crate::investigation::set_character_case_site(' in arrival
selector = contents['crates/adventuresim-stdb-module/src/disease/exposure_sources.rs']
assert b'pub(super) fn for_settlement(' in selector and b'.local_problem_authority()' in selector
assert b'scoped: true' in selector
illness = contents['crates/adventuresim-stdb-module/src/disease.rs']
plan = illness.split(b'pub fn plan_party_disease_interval(', 1)[1].split(b'#[cfg(test)]', 1)[0]
assert b'if let Some(settlement_id) = character.current_settlement_id' in plan
assert b'for_settlement(ctx, &settlement_id)?' in plan
assert b'crate::outbreak::exposure_windows(ctx, &source_id, *id, from, to)' in plan
assert b'&environmental,' in plan
assert b'owned_plan = plan_party_disease_interval(ctx, &[character_id], requested, allow_healing)?' in illness
advancement = contents['crates/adventuresim-stdb-module/src/time/advancement.rs']
assert b'crate::disease::clip_elapsed_for_disease(ctx, character_id, injury_limit, false)?' in advancement
assert b'let alive = advance_character_time(ctx, character_id, safe)?' in advancement
act = contents['crates/adventuresim-stdb-module/src/investigation/actions.rs']
assert b'advance_investigation_time(ctx, *member_id, effect_minutes)?' in act
cost = contents['crates/adventuresim-core/src/investigation_action.rs']
assert b'K::InspectSite => StrategicCost {\n            minutes: 45,' in cost
assert b'.max(15),' in cost

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 13
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
for number, original_indices in ((263, [1148]), (264, [1149, 1150, 1151, 1152])):
    name = f'segovb/READ-L12-{number}.json'; receipt = json.loads(by_path[name])
    assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == original_indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert receipt['inherited_disease1093_read_qualification_unchanged'] is True
    assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
    for row in receipt['rows']:
        assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
    verify = json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert verify['receipt_sha256'] == sha(by_path[name])
    assert verify['state_sha256'] == sha(by_path[f'segovb/STATE-L12-{number}.json'])

history = load('ASSESSMENT-L01-HISTORICAL-1148-1151.json')['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1148, 1152))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1148-1151.json'
        assert role['status'] == 'full-historical-body-trace-open'
        if row['original_index'] == 1148:
            assert row['prior_role_status'] == 'bounded-current-trace-only'
            assert row['prior_current_observations'] == role['current_observations']
            assert row['prior_current_qualification'] == role['qualification']
queue = load('RECONCILIATION-059.json')['entries']
for i, name in ((220, ids[1]), (953, 'Q-L12-e3dae8aea9e80e1a3d05')):
    assert queue[i]['contract_id'] == name and queue[i]['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for cid in ids:
    assert len([x for x in ledger if x['id'] == cid and x['record'] == 'ASSESSMENT-L12-OUTBREAK-SITE-EXPOSURE-95E.json']) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified outbreak source selection, earlier attribution and archived roles; no product ran')
