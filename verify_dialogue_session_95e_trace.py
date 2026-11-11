"""Verify dialogue session source evidence and scoped historical attribution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009';name='ASSESSMENT-L12-DIALOGUE-SESSION-95E.json';doc=load(name)
ids=['SEGOVB-L12-DIALOGUE-LANGUAGE-CATEGORY-TEXT','SEGOVB-L12-COND-SESSION-NPC-REUSE-BINDING']
assert doc['source_revision']==rev and doc['lens']==12 and [r['id'] for r in doc['candidates']]==ids
tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta,path=line.split('\t',1);tree[path]=meta.split()[2]
contents={}
for row in doc['current_sources']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']==tree[row['path']]
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines']
    assert row['revision']==rev and all(1<=lo<=hi<=row['lines'] for lo,hi in row['ranges'])
    contents[row['path']]=raw
session=contents['crates/adventuresim-stdb-module/src/strategic/dialogue_sessions.rs']
assert b'if conversation_id != npc.conversation_id' in session
existing=session.split(b'if let Some(existing) = ctx.db.dialogue_session().id().find(&session_id)',1)[1].split(b'let id = session_id;',1)[0]
assert b'existing.conversation_id == conversation_id' in existing
assert b'existing.settlement_id == settlement_id' in existing and b'existing.location_id == location_id' in existing
assert b'require_live_dialogue_presence(ctx, &existing, character_id)' in existing
assert b'npc_actor_id' not in existing and b'npc_character_id' not in existing
assert b'actor_id: bound.character_id.to_string()' in session
schema=contents['crates/adventuresim-stdb-module/src/strategic/dialogue_schema.rs']
assert b'fn require_live_dialogue_presence(' in schema and b'for participant in npc_participants' in schema
assert b'npc.home_settlement_id != session.settlement_id' in schema
resident=contents['crates/adventuresim-stdb-module/src/settlement_population/resident_drafts.rs']
assert b'resident_seed(settlement_id, "residences", ordinal)' in resident
assert b'(0, "householder", "resident", DefaultPresence::Default)' in resident
assert b'"domestic worker",\n            "neighbor"' in resident
assert b'if !character_ids.insert(draft.character_id())' in resident
persist=contents['crates/adventuresim-stdb-module/src/settlement_population/resident_persistence.rs']
assert b'let provider = draft.business_id.is_some() || draft.service.is_some();' in persist
assert b'if !provider {\n        "local-resident"' in persist
assert b'Schedule::Day => (360, 1200)' in persist and b'Schedule::Evening => (720, 1380)' in persist
assert b'Schedule::Early => (240, 960)' in persist and b'Schedule::Provider => (0, MINUTES_PER_DAY as u16)' in persist
assert b'location_id: draft.location.clone()' in persist
web=contents['crates/strategic-web/src/routes/dialogue.rs']
assert b'let session_id = format!("dialogue:{character_id}:{nonce}")' in web
assert b'json!(npc_actor_id)' in web and b'build_view(state, character_id, session_id)' in web
provenance=contents['crates/adventuresim-stdb-module/src/strategic/dialogue_provenance.rs']
assert b'("fluent", true)' in provenance and b'("limited", true)' in provenance and b'("poor", false)' in provenance
assert b'FactKey::ParticipantLanguageCompatibility' in provenance and b'FactValue::Text(compatibility.into())' in provenance
authoring=contents['crates/adventuresim-dialogue/src/authoring_schema.rs']
assert b'Self::ParticipantSex { .. } => matches!(value, FactValue::Sex { .. })' in authoring
assert b'_ => !matches!(value, FactValue::Sex { .. })' in authoring
validation=contents['crates/adventuresim-dialogue/src/condition_validation.rs']
assert b'if !key.authoring_value_is_valid(equals)' in validation
runtime=contents['crates/adventuresim-dialogue/src/lib.rs']
assert b'Condition::Fact { key, equals } => self.facts.get(key) == Some(equals)' in runtime
assert b'"participant_language_compatibility"' in contents['content/dialogue/services.yaml']

transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records=doc['recovered_historical_records'];assert len(records)==23
by={}
for row in records:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());entry=transport[row['original_path']]
    assert row['sha256']==entry['sha256'] and row['git_blob']==entry['git_blob'] and row['bytes']==entry['bytes']
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if not row['redacted_local_path_occurrences']:assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    else:assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    by[row['original_path']]=raw
for number,indices in ((292,[1215]),(293,[1216]),(294,[1217,1218])):
    receipt_name=f'segovb/READ-L12-{number}.json';receipt=json.loads(by[receipt_name])
    assert receipt['lens']==12 and [r['original_index'] for r in receipt['rows']]==indices
    assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False and sha(by[receipt['plan']])==receipt['plan_sha256']
    for row in receipt['rows']:
        assert sha(by[row['registration']])==row['registration_sha256']
        assert row['fresh_personal_body_reads']==row['assessed_ranges'] and row['other_lens_verdict_transfer'] is False
    check=json.loads(by[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256']==sha(by[receipt_name])
    assert check['state_sha256']==sha(by[f'segovb/STATE-L12-{number}.json'])
granular=load('baseline-059/queue.json')['entries'][232]['granular_contract_views'][0]
assert granular['lens']==2 and granular['body_or_population_credit']==0
assert granular['contract_update']['new_accepted_unique_count']==0
assert granular['contract_update']['other_lens_coverage_transferred'] is False
assert sha(by[granular['manifest']['path']])==granular['manifest']['sha256']
for row in granular['exact_source_identities_and_assessed_ranges']:
    assert row['fresh_personal_ranges']==[] and row['body_or_population_credit_from_adoption']==0
    for key in ('receipt','registration','inherited_same_lens_qualification'):
        ref=row[key];assert sha(by[ref['path']])==ref['sha256']

histname='ASSESSMENT-L01-HISTORICAL-1215-1218.json';history=load(histname)['historical_source_roles']
assert [r['original_index'] for r in history]==list(range(1215,1219));roles=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read()
        assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']];assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
q=load('RECONCILIATION-059.json')['entries']
for index,identity in ((231,ids[0]),(232,ids[1]),(983,'Q-L12-7dd0e469d0d340178744'),(984,'Q-L12-f21ca3d1e45d6ba6707c'),(985,'Q-L12-70906337724f64113c49')):
    assert q[index]['contract_id']==identity and q[index]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches=[r for r in ledger if r['id']==identity];assert len(matches)==1 and matches[0]['record']==name
progress=load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_string_traces']
assert progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0
assert progress['new_current_ledger_candidates']==len(ledger)
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified dialogue session trace and scoped historical attribution; no product ran')
