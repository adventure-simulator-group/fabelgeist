#!/usr/bin/env python3
"""Verify selected source and recovery evidence, without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();a=load('ASSESSMENT-SIMULATION-BOUNDARIES-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1270-1275.json');assert a['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);source[x['path']]=raw.decode()
b='crates/adventuresim-strategic-sim/src/live_core/';boot=source[b+'bootstrap.rs'];cfg=source[b+'config.rs'];failure=source[b+'failure.rs'];coverage=source[b+'quest_coverage.rs'];events=source[b+'events.rs'];lock=source['Cargo.lock']
for label in ('direct_contract_continuation','direct_contract','generated_open_case','generated_discovery','activity'):assert f'"{label}"' in boot
assert 'let quest_selected = quest_path != "activity"' in boot and 'match quest_path {' in boot
assert 'Some("localhost" | "127.0.0.1" | "::1")' in cfg and '[::1]' in cfg
assert 'url = "2.5"' in source['crates/adventuresim-strategic-sim/Cargo.toml'] and 'use url::Url;' in source[b+'mod.rs'] and 'ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed' in lock
assert 'MAX_CORE_TRACE_EVENTS: usize = 100_000' in cfg and 'if self.trace.len() < MAX_CORE_TRACE_EVENTS' in failure and 'self.semantic_event_keys.push(semantic)' in failure
assert 'runner.semantic_event_keys.iter().any' in boot and 'coverage.direct_accepted' in coverage and 'trace_truncated' in source[b+'report.rs']
assert 'CoreLoopEventSemanticKey' in events and 'CoreLoopEventSubject::DirectContract' in events
archive=tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz');roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1270,1276))
for x in h['historical_source_roles']:
 raw=archive.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]];assert roles[x['original_index']]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1270-1275.json'
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob'])
q=load('RECONCILIATION-059.json')['entries']
for i,ident in [(249,'SEGOVB-L12-LIVE-QUEST-PATH-TEXT-DISPATCH'),(250,'SEGOVB-L12-LOOPBACK-IPV6-HOST-TEXT-MISMATCH'),(251,'SEGOVB-L12-COND-QUEST-COVERAGE-KEYS-TRACE-CAP')]:assert q[i]['contract_id']==ident and q[i]['current_reconciliation'].startswith('partial ')
baseline=load('BASELINE-95E-20261110.json');assert all(sum(x['id']==c['id'] for x in baseline['newly_assessed_ledger']['entries'])==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(baseline['newly_assessed_ledger']['entries'])
print('Verified six historical bodies, selected source chains and historical reconciliation; no product execution.')
