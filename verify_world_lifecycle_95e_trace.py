#!/usr/bin/env python3
"""Verify preserved evidence and static source selections, without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();a=load('ASSESSMENT-WORLD-EVENT-LIFECYCLE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1265-1269.json');assert a['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);source[x['path']]=raw.decode()
world=source['crates/adventuresim-stdb-module/src/world_event.rs'];runner=source['crates/adventuresim-strategic-sim/src/lifecycle/runner.rs'];privacy=source['crates/adventuresim-strategic-sim/src/lifecycle/privacy.rs'];sampling=source['crates/adventuresim-strategic-sim/src/lifecycle/runner/sampling.rs']
for tag in ('reputation','offense','local_problem','case_participant','infection_episode'):assert f'("{tag}",' in world
assert 'let identity = consequence_identity(consequence);' in world and 'identities.insert(identity)' in world
for field in ('formal_route_requires_opposite_sexes','formal_route_requires_father_approval','informal_route_covers_father_disapproval','informal_route_covers_same_sex_couple','secrecy_checks_required'):
 assert f'{field}: true,' in runner and f'&& c.{field}' in runner
assert 'Does not invoke SpacetimeDB reducers, persistence, subscriptions, or the browser UI.' in runner
assert 'self.now.day_start().saturating_add_days(1)' in runner and 'lifecycle_entropy(seed, "conception", trial.ordinal)' in runner
assert 'StreamId::new("simulation.lifecycle")' in sampling and 'domain.as_bytes()' in sampling
assert 'serde_json::to_string(value)?' in privacy and 'lower.contains(**fragment)' in privacy
archive=tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz');roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1265,1270))
for x in h['historical_source_roles']:
 raw=archive.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]];role=roles[x['original_index']];assert role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1265-1269.json' and role.get('current_observations',[])==x['prior_current_observations']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
for n,eof in ((324,1188),(326,1325)):
 first=records[f'segovb/PARTIAL-L12-{n}-001.json'];second=records[f'segovb/PARTIAL-L12-{n}-002-EOF.json'];assert first['fresh_personal_body_reads']==[[1,600]] and first['unread_tail']==[[601,eof]];assert second['fresh_personal_body_reads']==[[601,eof]] and second['unread_tail']==[]
q=load('RECONCILIATION-059.json')['entries']
for i,ident in [(247,'SEGOVB-L12-WORLD-EVENT-CONSEQUENCE-TEXT-KEY'),(248,'SEGOVB-L12-COND-LIFECYCLE-POLICY-METRIC-LITERALS'),(1057,'Q-L12-6605957d34805b2ecd9a')]:assert q[i]['contract_id']==ident and q[i]['current_reconciliation'].startswith('partial ')
b=load('BASELINE-95E-20261110.json');assert all(sum(x['id']==c['id'] for x in b['newly_assessed_ledger']['entries'])==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(b['newly_assessed_ledger']['entries'])
print('Verified five historical bodies, preserved partials and static world-event/lifecycle source selections; no product execution.')
