"""Check archived incident-handoff source and historical evidence only."""
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
name = 'ASSESSMENT-INCIDENT-COMBAT-HANDOFF-95E.json'
doc = load(name)
ids = ['REVIEW-INCIDENT-COMBAT-AUTHORITY-HANDOFF-001', 'SEGOVB-L12-ACTIVITY-INCIDENT-KIND-TEXT-DISPATCH']
assert doc['source_revision'] == revision and doc['lenses'] == [1, 12]
assert [c['id'] for c in doc['candidates']] == ids

tree = {}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1); tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == revision and all(1 <= a <= b <= row['lines'] for a,b in row['ranges'])
    contents[row['path']] = raw
incident = contents['crates/adventuresim-stdb-module/src/strategic/incidents.rs']
creation = incident.split(b'fn create_strategic_incident(',1)[1].split(b'fn maybe_trigger_religious_incident(',1)[0]
for x in (b'incident.status == IncidentStatus::Pending', b'incident.source_id == source_id',
          b'case_id: incident_id.value.clone()', b'case_site_authority().insert(site.clone())',
          b'materialize_hostile_group(', b'strategic_incident().insert(StrategicIncident',
          b'set_character_case_site(', b'party.current_case_site_id = Some('):
    assert x in creation
assert b'case_authority()' not in creation and b'investigation_lead()' not in creation
activity = incident.split(b'pub(crate) fn maybe_trigger_activity_incident(',1)[1].split(b'pub fn surrender_to_authority(',1)[0]
for x in (b'"raiding"', b'"thievery"', b'"carousing_disorder"', b'"authority_arrest"',
          b'let (incident_kind, kind_key) = match kind',
          b'"raiding" => (IncidentKind::RaidingRetaliation, "raiding")',
          b'"thievery" => (IncidentKind::ThieveryDiscovery, "thievery")',
          b'"carousing_disorder" => (IncidentKind::CarousingDisorder, "carousing_disorder")',
          b'_ => (IncidentKind::AuthorityArrest, "authority_arrest")',
          b'activity_incident_source_id(', b'snapshot_arrest_charges('):
    assert x in activity
assert b'kind: IncidentKind::Religious' in incident
owner=contents['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
materialize=owner.split(b'fn materialize_hostile_group(',1)[1].split(b'fn hostile_group_authority_row(',1)[0]
assert b'hostile_group_authority().insert(group.clone())' in materialize
assert b'materialize_context_roster(' in materialize
assert b'case_authority()' not in materialize and b'investigation_lead()' not in materialize
world=contents['crates/adventuresim-stdb-module/src/world_actor.rs']
roster=world.split(b'pub(crate) fn materialize_context_roster(',1)[1].split(b'pub(crate) fn rebind_road_cast_to_strategic_encounter(',1)[0]
assert b'character_context_membership()' in roster and b'case_authority()' not in roster
assert b'investigation_lead()' not in roster
model=contents['crates/adventuresim-stdb-module/src/investigation/model.rs']
occupancy=model.split(b'pub(crate) fn set_character_case_site(',1)[1].split(b'\n}',1)[0]
assert b'character_case_site_occupancy()' in occupancy
assert b'case_authority()' not in occupancy and b'investigation_lead()' not in occupancy
provenance=contents['crates/adventuresim-stdb-module/src/investigation/sites/provenance.rs']
for marker in (b'pub(crate) fn case_site_provenance_view(',b'pub(crate) fn case_site_provenance_reducer('):
    section=provenance.split(marker,1)[1].split(b'\n}',1)[0]
    assert b'case_authority().id().find(&site.case_id)?' in section
sites=contents['crates/adventuresim-stdb-module/src/investigation/sites.rs']
pins=sites.split(b'pub fn backend_case_site_pins(',1)[1].split(b'\n}',1)[0]
assert b'investigation_lead()' in pins and b'case_site_provenance_view(ctx, &site)?' in pins
exact=sites.split(b'pub(crate) fn exact_case_site_for_observer_at(',1)[1].split(b'\n}',1)[0]
assert b'case_site_provenance_reducer(ctx, &site)?' in exact and b'investigation_lead()' in exact
assert b'lead.destination_stage.is_exact()' in exact
custody=contents['crates/adventuresim-stdb-module/src/strategic/custody_objectives.rs']
bound=custody.split(b'pub(crate) fn ensure_bound_mission_authority(',1)[1].split(b'let expression:',1)[0]
assert b'exact_case_site_for_observer(ctx, observer_character_id, case_site.id.as_str()).is_none()' in bound
assert b'case_authority()' in bound
mission=contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
auto=mission.split(b'pub fn autoresolve_mission(',1)[1].split(b'if mission.status',1)[0]
assert b'require_party_ready(ctx, &party_id)?' in auto and b'ensure_bound_mission_authority(' in auto
tactical=contents['crates/adventuresim-stdb-module/src/tactical.rs']
entry=tactical.split(b'pub fn request_tactical_server(',1)[1].split(b'if mission.status',1)[0]
assert b'case_site_provenance_reducer(ctx, &case_site)' in entry
assert b'None => return Err("Case-site combat provenance is invalid or ambiguous".into())' in entry
assert entry.index(b'case_site_provenance_reducer') < entry.index(b'ensure_bound_mission_authority')
web=contents['crates/strategic-web/src/routes/quests.rs']
page=web.split(b'async fn render_quest_location(',1)[1].split(b'let manual_contract',1)[0]
assert b'backend_case_site_pins' in page and b'let Some(site) = known_site.as_ref() else' in page
assert b'StatusCode::NOT_FOUND' in page
webentry=contents['crates/strategic-web/src/routes/missions.rs'].split(b'async fn enter_mission(',1)[1].split(b'async fn mission_status(',1)[0]
assert b'party.active_contract_id' in webentry and b'backend_case_site_pins' in webentry
assert b'site.case_id == contract.case_id' in webentry
spec=contents['crates/adventuresim-stdb-module/src/strategic/tests/combat_party.rs']
assert b'incidents_own_sources_sites_and_lifecycle_without_quest_side_effects' in spec
assert b'assert!(!create.contains("ctx.db.quest()"))' in spec
sim=contents['crates/adventuresim-strategic-sim/src/live_core/expedition.rs']
assert b'surrender_affordable_authority_arrest' in sim and b'surrender_to_authority_then' in sim
assert b'observed_activity_site_origins' in sim

transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records=doc['recovered_historical_records'];assert len(records)==12;by={}
for row in records:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());entry=transport[row['original_path']]
    assert (row['sha256'],row['git_blob'],row['bytes'])==(entry['sha256'],entry['git_blob'],entry['bytes'])
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if row['redacted_local_path_occurrences']:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else: assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    by[row['original_path']]=(row,json.loads(raw))
def original_sha(path): return by[path][0]['sha256']
for number,indices in ((297,[1221,1222]),(298,[1223,1224])):
    key=f'segovb/READ-L12-{number}.json';receipt=by[key][1]
    assert receipt['lens']==12 and [r['original_index'] for r in receipt['rows']]==indices
    assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False and original_sha(receipt['plan'])==receipt['plan_sha256']
    for row in receipt['rows']:
        assert original_sha(row['registration'])==row['registration_sha256']
        assert row['fresh_personal_body_reads']==row['assessed_ranges'] and row['other_lens_verdict_transfer'] is False
    check=by[f'segovb/VERIFY-L12-{number}.json'][1]
    assert check['receipt_sha256']==original_sha(key)
    assert check['state_sha256']==original_sha(f'segovb/STATE-L12-{number}.json')
    assert by[key][1]['rows'][-1]['source_sha256']==load('COVERAGE-L01-ROLES.json')['roles'][indices[-1]]['historical_sha256']

histname='ASSESSMENT-L01-HISTORICAL-1221-1224.json';history=load(histname)['historical_source_roles']
assert [row['original_index'] for row in history]==[1221,1222,1223,1224]
roles=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read()
        assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']]
        assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
        assert role.get('current_observations',[])==row['prior_current_observations']
q=load('RECONCILIATION-059.json')['entries']
assert q[237]['contract_id']==ids[1] and q[237]['current_reconciliation'].startswith('partial ')
assert q[989]['contract_id']=='Q-L12-56543a94b7ecbd1f18bc' and q[989]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matching=[r for r in ledger if r['id']==identity];assert len(matching)==1 and matching[0]['record']==name
progress=load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_behavior_traces'] and name in progress['post_transition_bounded_string_traces']
assert progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0
assert progress['new_current_ledger_candidates']==len(ledger)
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified incident handoff and attributed historical source evidence; no product ran')
