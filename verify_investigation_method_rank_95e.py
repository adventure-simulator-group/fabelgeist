#!/usr/bin/env python3
"""Verify source identities and qualified historical evidence, not product behavior."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-INVESTIGATION-METHOD-RANK-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1282.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'] and x['revision']==rev;assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/';policy=s[base+'investigation_policy.rs'];rank=s[base+'encounter_policy.rs'];vocab=s['crates/adventuresim-core/src/investigation_action/vocabulary.rs'];cap=s['crates/adventuresim-stdb-module/src/investigation/capabilities.rs'];projection=s['crates/adventuresim-stdb-module/src/investigation/projections.rs'];generated=s[base+'generated_cases.rs'];camp=s['crates/adventuresim-stdb-module/src/time/camp_rest.rs'];config=s[base+'config.rs'];boot=s[base+'bootstrap.rs'];rules=s['crates/AGENTS.md']
assert b'pub enum InvestigationActionKind' in vocab and b'impl FromStr for InvestigationActionKind' in vocab
assert b'kind: action::InvestigationActionKind' in cap and b'method: kind.stable_id().into()' in cap
assert b'pub method: String' in projection and b'capability.method.parse::<InvestigationActionKind>().ok()?' in projection and b'method: capability.method' in projection
assert b'generated_method_skill_fit(profile, &action.method)' in policy and b'actions.sort_by' in policy and b'let hours = match method' in rank and b'_ => 0.0' in rank
assert b'sort_generated_actions(profile, &mut actions)' in generated
assert b'projected_investigation_wait_minutes' in policy and b'NightWindow' in policy and b'ContactScheduleWindow' in policy and b'at_settlement && wait_minutes >= 60' in generated and b'map at a settlement' in camp
assert b'profiles.len().div_ceil(party_size)' in policy and b'.expect("party target capacity covers every profile")' in policy
assert b'!(2..=8).contains(&self.party_size)' in config and b'config.validate()?' in boot and b'balanced_party_groups(&runner.profiles, config.party_size as usize)' in boot and b'Never use `unwrap` or `expect` outside tests.' in rules
x=h['historical_source_roles'][0];assert x['original_index']==1282 and x['lines']==528 and x['new_personal_read_ranges']==[[1,528]]
roles=load('COVERAGE-L01-ROLES.json')['roles'];assert roles[1282]['status']=='full-historical-body-trace-open' and roles[1282]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1282.json'
with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:
 raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==528 and raw==policy
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};rec={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);rec[x['original_path']]=json.loads(raw)
receipt=rec['segovb/READ-L12-337.json'];assert receipt['rows'][0]['original_index']==1282 and receipt['rows'][0]['fresh_personal_body_reads']==[[1,528]] and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
q0=rec['adler/QUALIFICATION-L12-234-READLOG.json'];assert q0['persisted_prior_read_ranges']==[[1,1200]] and q0['additional_inherited_read_attestation']==[[1201,1500],[1501,1800]] and q0['unread_ranges']==[[1801,2950]] and q0['completion_credit']==0
pause=rec['evidence/continuation-segovb/audit-resume-20261027/lens12/HANDOFF-paused-adler-001.json'];assert pause['root_adopted_frontier']['next_original']==1079 and pause['in_flight']['completion_credit']==0 and pause['in_flight']['not_read']==[[1801,2950]]
q=load('RECONCILIATION-059.json')['entries'];o=load('baseline-059/queue.json')['entries']
for i in [1066,1074]:assert q[i]['contract_id']==o[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ')
b=load('BASELINE-95E-20261110.json');ledger=b['newly_assessed_ledger']['entries'];assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified role 1282, method and grouping source, and inherited partial-read qualifications; no product execution.')
