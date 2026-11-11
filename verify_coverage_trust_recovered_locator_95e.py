#!/usr/bin/env python3
"""Verify source identities and distinct historical exports; execute no product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();a=load('ASSESSMENT-COVERAGE-TRUST-RECOVERED-LOCATOR-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1284-1289.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed';s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'] and x['revision']==rev;assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/';coverage=s[base+'quest_coverage.rs'];report=s[base+'report.rs'];config=s[base+'config.rs'];boot=s[base+'bootstrap.rs'];policy=s[base+'policy.rs'];main=s['crates/adventuresim-strategic-sim/src/main.rs'];err=s[base+'error.rs'];core=s['crates/adventuresim-core/src/reducer_error.rs'];native=s['crates/adventuresim-stdb-module/src/investigation/actions.rs'];rules=s['crates/AGENTS.md']
validator=coverage.split(b'pub fn validate_quest_coverage(',1)[1].split(b'pub fn write_quest_coverage_failure(',1)[0]
assert b'.ok_or_else(|| QuestCoverageFailure::new(QuestCoverageMetric::FixtureProvenance))?' in validator and b'report.final_agents.iter().any(|agent| !agent.alive)' in validator
assert b'coverage.direct_contract_id' not in validator and b'coverage.generated_case_id' not in validator and b'coverage.direct_party_id' not in validator and b'report.final_agents.is_empty()' not in validator
for token in [b'coverage.direct_accepted',b'coverage.direct_traveled',b'coverage.direct_encountered',b'coverage.direct_reported',b'coverage.generated_intake',b'coverage.generated_discovered',b'coverage.generated_completed',b'metrics.quests_attempted >= 2']:
 assert token in validator
assert b'pub struct CoreLoopReport' in report and b'pub final_agents: Vec<FinalAgentState>' in report and b'!(2..=32).contains(&self.population)' in config and b'let character_ids = (0..config.population)' in boot and b'.collect::<Result<Vec<_>, String>>()?' in boot and b'validate_quest_coverage(&report)' in main
assert b'fn reallocate_disabled_crime_to_labor' in policy and b'.expect("valid daily schedule crime allocation")' in policy and b'.expect("valid daily schedule labor allocation")' in policy and b'.expect("strategic day length must fit the persisted schedule width")' in policy and b'Never use `unwrap` or `expect` outside tests.' in rules
assert b'pub const MINUTES_PER_DAY: u64 = 24 * 60;' in s['crates/adventuresim-world-schema/src/calendar.rs']
assert b'coded_reducer_error(' in native and b'pub fn parse_reducer_error(value: &str)' in core and b'code: parse_reducer_error(&detail)' in err and b'Ok(Err(detail)) => Err(CoreLoopError::reducer_rejected(operation, detail))' in policy and b'Err(_) => Err(CoreLoopError::reducer_timed_out(operation))' in policy
roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1284,1290))
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]] and roles[x['original_index']]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1284-1289.json' and raw==s[x['source_id'].split(':',1)[1]]
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};rec={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);rec[x['original_path']]=json.loads(raw)
for n in range(338,342):
 receipt=rec[f'segovb/READ-L12-{n}.json'];assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
loc=rec['segovb/QUALIFICATION-L12-ORIGINAL1289-LOCATOR.json'];assert loc['historical_locator_proof_body_not_read'] is True and loc['new_population_credit']==0 and loc['registered_required_original_role_scope']==[[1,101]]
older={}
for x in a['recovered_older_774_records']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and x['older_publication_revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';older[x['path_in_774']]=raw
proof_manifest=load(a['older_774_recovery']['git_object_proof']);objects={}
for x in proof_manifest['objects']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and hashlib.sha1(x['type'].encode()+b' '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==x['git_object'];objects[x['git_object']]=(x['type'],raw)
assert objects[proof_manifest['commit']][0]=='commit' and objects[proof_manifest['commit']][1].splitlines()[0]==b'tree '+proof_manifest['root_tree'].encode()
def entries(raw):
 out={};pos=0
 while pos<len(raw):
  sp=raw.index(b' ',pos);nul=raw.index(b'\0',sp);out[raw[sp+1:nul].decode()]=(raw[pos:sp].decode(),raw[nul+1:nul+21].hex());pos=nul+21
 assert pos==len(raw);return out
for x in proof_manifest['paths']:
 tree=proof_manifest['root_tree'];parts=x['path'].split('/')
 for i,part in enumerate(parts):
  kind,raw=objects[tree];assert kind=='tree';mode,oid=entries(raw)[part]
  if i==len(parts)-1: assert mode=='100644' and oid==x['git_blob'] and blob(older[x['path']])==oid
  else: assert mode in ('40000','040000');tree=oid
prefix='handoff/774/';outer=json.loads(older[prefix+'MANIFEST.json']);outer_lookup={x['path']:x for x in outer['files']};package=json.loads(older[prefix+'adler-original/PACKAGE-MANIFEST.json']);package_lookup={x['path']:x for x in package['entries']}
for path,raw in older.items():
 relative=path[len(prefix):]
 if relative!='MANIFEST.json': assert outer_lookup[relative]['sha256']==sha(raw) and outer_lookup[relative]['bytes']==len(raw)
proofpath='audit/continuation/rig/source-scope-reconciliation-044.json';proof=older[prefix+'adler-original/'+proofpath];e=package_lookup[proofpath];assert e['original_sha256']=='fe7e2e81a37b125318a22c961f2d9bead728e93d3721e1e3a4142ff1f62defa1' and e['exported_sha256']==sha(proof)=='1221f858b25f889ab7918895b1f4a3329fb4cb0614b8664531ffd821e7202a31' and e['original_sha256']!=e['exported_sha256']
entry=json.loads(proof)['new_variant_identity_and_range_map'][101];assert entry['source_id']=='pr-working:crates/adventuresim-strategic-sim/src/live_core/schema_types.rs' and entry['recorded_sha256']==roles[1289]['historical_sha256'] and len(entry['observations'])==2
for group,wanted,pointer,index in [('camera','CAM-PPS-SIMULATION-OBSERVATION-047','additional_actual_source_proofs',54),('rig','RIG-PPS-036','additional_fresh_sources',19)]:
 path='audit/records/final-acceptance-20261021/wave-01/producer-supplement-after-owner/'+group+'/traces.jsonl';raw=older[prefix+'adler-original/'+path];x=package_lookup[path];assert x['original_sha256']==x['exported_sha256']==sha(raw);matches=[json.loads(line) for line in raw.splitlines() if wanted.encode() in line];matches=[x for x in matches if x.get('id',x.get('trace_id'))==wanted];assert len(matches)==1 and matches[0][pointer][index]['sha256']==roles[1289]['historical_sha256'] and matches[0][pointer][index]['read_ranges']==[[1,101]]
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [1071,1074,1076,1077,1078,1084]);b=load('BASELINE-95E-20261110.json');ledger=b['newly_assessed_ledger']['entries'];assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical bodies, 059 qualifications, older normalized locator and exact attributed receipts; no product execution.')
