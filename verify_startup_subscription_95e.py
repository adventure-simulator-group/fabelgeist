#!/usr/bin/env python3
"""Verify startup, SDK and historical evidence without running the product."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';pre='c100638b720a7281a5fc96f6a3c5c23d5dda85da';name='ASSESSMENT-STARTUP-SUBSCRIPTION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2072-2078.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2072:89,2073:63,2074:33,2075:120,2076:191,2077:75,2078:294};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations']==row.get('current_observations',[]) and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
  if i==2075:
   assert 'c100 predecessor' in item['prior_worklist_observation_revision_qualification']
   for prior in item['prior_current_observations']:
    doc=load(prior['assessment']);assert doc.get('revision',doc.get('current_revision',doc.get('source_revision')))==pre
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
native={}
for item in a['native_sdk_support']:
 manifest=load(item['archive_manifest']);archive_path=root/Path(item['archive_manifest']).parent/manifest['archive'];assert sha(archive_path.read_bytes())==manifest['sha256']
 with tarfile.open(archive_path,'r:gz') as archive:body=archive.extractfile(item['path']).read()
 assert sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['new_bounded_read_ranges']);native[item['path']]=body
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
startup=current[2075];stdb=current[2076];windows=current[2078];scene=sources['crates/adventuresim-tactical-core/src/scene_input.rs'];descriptor=sources['crates/adventuresim-tactical-core/src/scene_input/descriptor.rs'];main=sources['crates/adventuresim-tactical-server/src/main.rs'];generated=sources['crates/adventuresim-stdb-client/src/mod.rs'];window=sources['crates/adventuresim-building-generator/src/windows.rs'];sub=native['spacetimedb-sdk-2.6.1/src/subscription.rs'];conn=native['spacetimedb-sdk-2.6.1/src/db_connection.rs']
assert b'MAX_COMBAT_CONFIG_BYTES: u64 = 64 * 1024' in startup and b'std::fs::metadata(path)' in startup and b'std::fs::read_to_string(path)' in startup and startup.index(b'std::fs::metadata(path)')<startup.index(b'std::fs::read_to_string(path)')
assert b'MAX_SCENE_INPUT_BYTES: u64 = 32 * 1024 * 1024' in descriptor and b'let length = fs::metadata(path)?.len()' in scene and b'serde_json::from_slice(&fs::read(path)?)' in scene
assert b'.on_error(|ctx, error|' in stdb and b'SpacetimeDB subscription failed' in stdb and b'.add_query(|query| query.from.connected_players())' in stdb and b'connected_players.lock().unwrap().push(row.clone())' in stdb
assert b'fn setup_stdb_callbacks(conn: Res<SpacetimeDb>)' in main and b'conn.subscribe_connected_players();' in main and b'run_if(resource_added::<SpacetimeDbReady>)' in main
assert b'fn subscription_error(' in sub and b'self.subscriptions.remove(&query_set_id)' in sub and b'callback(ctx, ctx.event().clone().unwrap())' in sub and b'handle: handle.clone()' in sub
assert b'ParsedMessage::SubscriptionError { query_set_id, error }' in conn and b'inner.subscriptions.subscription_error(&ctx, query_set_id);\n                Ok(())' in conn and b'inner.subscriptions.register_subscription(query_set_id, handle.clone())' in conn and b'fn unsubscribe(self)' in generated
assert b'solid.centre.metres() - Vec3::new(tangent.x, 0.0, tangent.y) * width * 0.5' in window and b'opening_centre_metres: window.closed_centre' in windows
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((698,[2072,2073,2074,2075,2076]),(699,[2077,2078,2079])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index'];expected=lengths.get(i,646);assert row['fresh_personal_body_reads']==[[1,expected]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
old=json.loads(records['adler/READ-L11-322.json']);later=json.loads(records['adler/READ-L11-324.json']);last=json.loads(records['adler/READ-L11-325.json'])
for i in (2075,2077):
 before=next(x for x in old['connected_reinspection'] if x['original_index']==i);after=next(x for x in later['sources'] if x['original_index']==i);assert before['fresh_reads']==[[1,lengths[i]]] and before['additional_original_frontier_credit']==0 and after['fresh_reads']==[] and after['registration_sha256']==before['registration_sha256'] and after['sha256']==before['source_sha256']==sha(current[i])
for i in (2072,2073,2074,2076):
 row=next(x for x in later['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
row=next(x for x in last['sources'] if x['original_index']==2078);assert row['fresh_reads']==[[1,294]] and row['sha256']==sha(current[2078]);assert [x['original_index'] for x in last['sources'] if x['original_index'] in (2079,2080,2081,2082)]==[2079,2080,2081,2082]
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (278,1655,1778,1779):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified seven historical roles, preserved reuse and delegated attribution, and source-bound startup and SDK traces; no product execution.')
