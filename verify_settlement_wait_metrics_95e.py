#!/usr/bin/env python3
"""Verify preserved source and earlier partial-read limits; execute no product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();a=load('ASSESSMENT-SETTLEMENT-WAIT-METRICS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1290-1291.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed';s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'] and x['revision']==rev;assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/';settlement=s[base+'settlement.rs'];activity=s[base+'settlement_activity.rs'];rest=s['crates/adventuresim-stdb-module/src/time/settlement_rest.rs'];repair=s['crates/adventuresim-stdb-module/src/repair.rs'];durability=s['crates/adventuresim-core/src/durability.rs'];economy=s['crates/adventuresim-world-schema/src/economy.rs'];shared=s['crates/adventuresim-core/src/settlement_economy.rs'];report=s[base+'report.rs']
assert b'self.metrics.repair_wait_minutes += wait;' in settlement and b'remaining = ready_at.elapsed_since(calendar_minute(&current));' in settlement and b'format!("minutes={wait};orders={}"' in settlement
assert b'self.metrics.activity_days += 1;' in settlement and b'format_activity_detail(activity_diagnostic, &before, &after)' in settlement and b'elapsed_delta={}' in activity
assert b'let elapsed = settled.elapsed;' in rest and b'clip_elapsed_for_disease(ctx, character_id, injury_limit, true)' in rest and b'.map(|_| ())' in rest and b'if terminal.is_some() || !settled.alive' in rest
assert b'pub repair_wait_minutes: u64' in report and b'pub activity_days: u32' in report
assert b'pub enum RepairService' in durability and b'pub fn parse(value: &str)' in durability and b'repair_service(&service)?' in repair and b'service.to_string()' in settlement
assert b'SettlementEconomyProfile::stage_placeholder()' in activity and b'.validate()' in activity and b'pub fn validate(&self)' in economy and b'pub fn npc_location_is_navigable' in shared and b'pub fn storefront_stocks' in shared
roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==[1290,1291]
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]] and roles[x['original_index']]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1290-1291.json' and raw==s[x['source_id'].split(':',1)[1]]
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};rec={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);rec[x['original_path']]=json.loads(raw)
for n,span,tail in [(1,[1,600],[[601,2043]]),(2,[601,1200],[[1201,2043]]),(3,[1201,1800],[[1801,2043]]),(4,[1801,2043],[])]:
 name=f'segovb/PARTIAL-L12-342-{n:03d}'+('-EOF' if n==4 else '')+'.json';x=rec[name];assert x['fresh_personal_body_reads']==[span] and x['unread_tail']==tail and x['new_original_overlay_merged_shared_population_credit']==0 and x['other_lens_coverage_transferred'] is False
for n in [342,343]:assert rec[f'segovb/READ-L12-{n}.json']['accepted_unique_count_change']==0 and rec[f'segovb/READ-L12-{n}.json']['current_source_reconciled'] is False
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [1079,1081,1082]);b=load('BASELINE-95E-20261110.json');ledger=b['newly_assessed_ledger']['entries'];assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified two historical bodies, four prior partials and settlement metric sources; no product execution.')
