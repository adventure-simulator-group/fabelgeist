#!/usr/bin/env python3
"""Verify source identities, qualified prior reads and symbolic clock values only."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();a=load('ASSESSMENT-GENERATED-ROUTE-TIME-AUTHORITY-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1292.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed';s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'] and x['revision']==rev;assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/';survival=s[base+'survival.rs'];generated=s[base+'generated_cases.rs'];settlement=s[base+'settlement.rs'];thermal=s[base+'thermal_projection.rs'];native=s['crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs'];journey=s['crates/adventuresim-stdb-module/src/strategic/journey_camp.rs'];clock=s['crates/adventuresim-stdb-module/src/time/clock.rs'];departure=s['crates/adventuresim-stdb-module/src/time/settlement_rest.rs'];calendar=s['crates/adventuresim-world-schema/src/calendar.rs'];weather=s['crates/adventuresim-core/src/weather.rs'];camp=s['crates/adventuresim-stdb-module/src/time/camp_rest.rs'];condition=s['crates/adventuresim-stdb-module/src/condition.rs']
selected=generated.split(b'let mut followed_safe_wait = false;',1)[1].split(b'let result =\n                    reducer_call!(self, ReducerOperation::TravelToGeneratedCaseSite',1)[0]
assert b'DepartureReadiness::ReadyWithItinerary' in selected and b'self.configure_safe_departure_itinerary(' in selected and b'case_site_recovery_minutes;\n                                self.configure_safe_departure_itinerary(' in selected
assert b'self.wait_for_safe_departure_at_settlement(' in selected and b'validate_case_site_thermal_readiness_at(' in selected and b'followed_safe_wait = true;' in selected
assert b'Some(journey_start_minute_of_day)' in settlement and b'journey_start_minute_of_day.unwrap_or(if travel_at_night' in settlement
assert b'DEFAULT_JOURNEY_START_MINUTE_OF_DAY: u16 = 8 * 60' in s['crates/adventuresim-core/src/strategic_time.rs'] and b'DEFAULT_NIGHT_JOURNEY_START_MINUTE_OF_DAY: u16 = 20 * 60' in s['crates/adventuresim-core/src/strategic_time.rs']
assert b'party.journey_start_minute_of_day = journey_start.get();' in native and b'party.wilderness_canonical_anchor_minute.is_some()' in native
assert b'configured_starting_minute.unwrap_or(observed_starting_minute)' in survival and b'backend_character_times()' in survival and b'starting_minute.saturating_add_minutes(offset)' in thermal
assert b'party.wilderness_canonical_anchor_minute = Some(refresh_clock(ctx)?)' in departure and b'anchor.with_wrapped_time_of_day(' in departure and b'calculate_official_minute(' in clock
assert b'weather_at(' in native and b'departure_minute,' in native and b'party_wilderness_environment_minutes(&party)' in condition and b'environment_start.wrapping_day_offset(offset)' in condition
assert b'WEATHER_INTERVAL_MINUTES: u64 = 360' in weather and b'.period_index(WEATHER_INTERVAL_MINUTES)' in weather and b'map at a settlement' in camp
assert (23*60+120)//1440==1 and (23*60+120)%1440==60 and 1440//360==4
planned=s['crates/adventuresim-stdb-module/src/time/advancement.rs'].splitlines()[74:145];assert b'apply_weather_exposure' not in b'\n'.join(planned)
assert b'#[primary_key]\n    pub character_id: u64' in s['crates/adventuresim-stdb-module/src/settlement_population.rs']
x=h['historical_source_roles'][0];assert x['original_index']==1292 and x['lines']==2550 and x['new_personal_read_ranges']==[[1,2550]];roles=load('COVERAGE-L01-ROLES.json')['roles'];assert roles[1292]['status']=='full-historical-body-trace-open' and roles[1292]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1292.json'
with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:
 raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==2550 and raw==survival
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};rec={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);rec[x['original_path']]=json.loads(raw)
for n,span,tail in [(1,[1,600],[[601,2550]]),(2,[601,1200],[[1201,2550]]),(3,[1201,1800],[[1801,2550]]),(4,[1801,2400],[[2401,2550]]),(5,[2401,2550],[])]:
 name=f'segovb/PARTIAL-L12-344-{n:03d}'+('-EOF' if n==5 else '')+'.json';x=rec[name];assert x['fresh_personal_body_reads']==[span] and x['unread_tail']==tail and x['new_original_overlay_merged_shared_population_credit']==0 and x['other_lens_coverage_transferred'] is False
receipt=rec['segovb/READ-L12-344.json'];assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False and receipt['supporting_reads'][0]['fresh_personal_body_reads']==[[1,160]]
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [1083,1085]);b=load('BASELINE-95E-20261110.json');ledger=b['newly_assessed_ledger']['entries'];assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified historical role 1292, five earlier partials and source-bound route clocks; no product execution.')
