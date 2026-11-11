#!/usr/bin/env python3
"""Verify preserved source evidence and symbolic arithmetic without running product code."""
import gzip,hashlib,json,math,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-SETTLEMENT-LEISURE-REAPPLICATION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1250-1256.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);source[x['path']]=raw.decode()
module='crates/adventuresim-stdb-module/src/';core='crates/adventuresim-core/src/'
rest=source[module+'time/settlement_rest.rs'].split('fn rest_for_minutes(',1)[1].split('fn inn_stay_cost(',1)[0]
assert rest.count('crate::condition::apply_settlement_leisure_condition(')==1 and rest.count('apply_activity_outcomes(')==1
assert rest.index('crate::condition::apply_settlement_leisure_condition(')<rest.index('if terminal.is_some() || !settled.alive')<rest.index('let training_elapsed = elapsed;')<rest.index('apply_activity_outcomes(')
assert rest.count('core_schedule(&effective_schedule)')>=1 and 'starting_minute.saturating_add_minutes(elapsed)' in rest
activities=source[module+'time/activities.rs'];wrapper=activities.split('fn apply_activity_outcomes(',1)[1].split('fn apply_activity_outcomes_without_leisure(',1)[0];inner=activities.split('fn apply_activity_outcomes_inner(',1)[1].split('fn immediate_activity_schedule(',1)[0]
assert 'true,' in wrapper and 'if apply_leisure {' in inner and 'apply_settlement_leisure_condition(' in inner and 'apply_spouse_leisure_conception(' in inner
condition=source[module+'condition.rs'];assert 'settlement_leisure_outcome(schedule, elapsed_minutes, stats.calories_used)' in condition and 'retained + earned' in condition and 'apply_residence_leisure_morale(' in condition
calc=source[core+'strategic_schedule.rs'];assert 'BASELINE_FATIGUE_PER_DAY: f32 = 600.0' in calc and 'LEISURE_FATIGUE_RECOVERY_PER_HOUR: f32 = 100.0' in calc and 'LEISURE_MORALE_SCALE_FATIGUE: f32 = 200.0' in calc
allocation=source[core+'strategic_schedule/allocation.rs'];assert 'Reading remains unchanged' in allocation and 'schedule.allocated_minutes() > MINUTES_PER_DAY' in allocation
assert 600*60/1440==25 and 2*25==50
morale=4*(1-math.exp(-1800/200))/24;assert abs(morale-0.1666461)<1e-6 and abs(2*morale-0.3332922)<1e-6
archive=tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz');roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1250,1257))
for x in h['historical_source_roles']:
 raw=archive.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]];assert roles[x['original_index']]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1250-1256.json'
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob'])
assert a['candidates'][0]['id']=='REVIEW-SETTLEMENT-REST-LEISURE-REAPPLICATION-001';b=load('BASELINE-95E-20261110.json');assert sum(x['id']==a['candidates'][0]['id'] for x in b['newly_assessed_ledger']['entries'])==1;p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(b['newly_assessed_ledger']['entries'])
print('Verified seven historical bodies, selected source bytes and symbolic leisure arithmetic; no product execution.')
