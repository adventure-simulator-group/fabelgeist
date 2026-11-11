#!/usr/bin/env python3
"""Verify recovered source bytes and bounded audit records; execute no product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-SIMULATION-WAIT-QUEUE-LINKS-95E.json')
h=load('ASSESSMENT-L01-HISTORICAL-1281.json')
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines']
 assert x['revision']==rev and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges'])
 source[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/'
g=source[base+'generated_cases.rs']; survival=source[base+'survival.rs']
settle=source['crates/adventuresim-stdb-module/src/time/settlement_rest.rs'];camp=source['crates/adventuresim-stdb-module/src/time/camp_rest.rs']
report=source[base+'report.rs'];projection=source[base+'failure_projection.rs']
assert b'projected_investigation_wait_minutes(reason, wait_minutes)' in g
assert b'generated_investigation_wait_minutes\n            .saturating_add(u64::from(wait_minutes))' in g
assert b'wait_minutes={wait_minutes};mode={wait_mode}' in g
assert b'rest_at_camp_with_party_shelter(' in g and b'rest_at_settlement_hours_then(' in g
assert b'Ok(shelter)' in survival and b'pub generated_investigation_wait_minutes: u64' in report
assert b'pub fn rest_at_settlement_hours(' in settle and b'.map(|_| ())' in settle
assert b'clip_elapsed_for_disease(ctx, character_id, injury_limit, true)' in settle and b'let elapsed = settled.elapsed;' in settle
assert b'map at a settlement' in camp and b'let member_elapsed = settled.elapsed;' in camp and b'Ok(())' in camp
fixture=json.loads(source['crates/adventuresim-strategic-sim/fixtures/core-loop-failure-v9.json'])
assert fixture['schema_version']==9 and fixture['category']=='investigation_temporally_unavailable' and fixture['reason_code']=='investigation_night_window' and fixture['operation']=='perform_investigation_action'
assert fixture['quest_coverage'] is None and fixture['total_event_count']==1 and len(fixture['trace'])==1 and all(v==0 for v in fixture['metrics'].values())
assert b'Some(ReducerErrorCode::InvestigationNightWindow)' in projection and b'CoreLoopFailureCategory::InvestigationTemporallyUnavailable' in projection and b'CoreLoopFailureReason::InvestigationNightWindow' in projection
assert b'let operation = error.operation_kind();' in projection
roles=load('COVERAGE-L01-ROLES.json')['roles']; assert len(h['historical_source_roles'])==1
x=h['historical_source_roles'][0]; assert x['original_index']==1281 and x['lines']==2736 and x['new_personal_read_ranges']==[[1,2736]]
assert roles[1281]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1281.json' and roles[1281]['status']=='full-historical-body-trace-open'
with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:
 raw=ar.extractfile(x['sha256']).read(); assert sha(raw)==x['sha256'] and len(raw.splitlines())==2736 and raw==g
previous=load('ASSESSMENT-EXPEDITION-DIALOGUE-RULES-95E.json')
rec={x['original_path']:x for x in previous['recovered_historical_records']}
assert 'segovb/PARTIAL-L12-336-005-EOF.json' in rec and 'segovb/PARTIAL-L12-336-005-EOF-CORRECTED.json' in rec
q=load('RECONCILIATION-059.json')['entries'];original=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for link in a['queue_links']:
 i=link['queue_index']; ident=link['current_ledger_id']; assert q[i]['contract_id']==original[i]['contract_id']==link['historical_contract_id'] and q[i]['current_reconciliation'].startswith('partial ')
 assert sum(x['id']==ident for x in ledger)==1
 entry=next(x for x in ledger if x['id']==ident)
 assert any('/entries/'+str(i)+' via ASSESSMENT-SIMULATION-WAIT-QUEUE-LINKS-95E.json' in x for x in entry['historical_related_records'])
for i in [1055,1079]: assert q[i]['current_reconciliation'].startswith('partial ')
assert sum(x['id']=='Q-L12-ef9ca7cb50456af30916' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(ledger) and p['queue_records_semantically_reconciled']==0
print('Verified role 1281, selected source bytes, fixture comparison and bounded queue links; no product execution.')
