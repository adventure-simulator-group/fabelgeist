#!/usr/bin/env python3
"""Verify publication evidence only; do not execute product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-FILESYSTEM-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2259-2260.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2259,2261))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2259,2260)]==[53,944]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
fs=bodies[2260];manifest=bodies[2259];mhr=sources['crates/fabelgeist-mhr/src/model.rs'];image=sources['crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs'];rules=sources['crates/AGENTS.md'];lock=sources['Cargo.lock']
assert b'std::fs::read(path).map_err(|e| anyhow!("Failed to read file {}: {}", path, e))' in fs and b'std::fs::read(&self.path).map_err(|e| anyhow!("Read failed: {}", e))' in fs and b'.map_err(|e| anyhow!("Base64 decode failed: {}", e))' in fs
assert b'let response = reqwest::get(uri).await?' in fs and b'Ok(response.bytes().await?.to_vec())' in fs and b'if !resp.ok()' in fs and b'match fabelgeist_fs::read_bytes(&direct).await' in mhr and b'fabelgeist_fs::read_bytes(&nested).await' in mhr and b'fabelgeist_fs::read_bytes(&uri).await?' in image
assert b'format!("file://{}", path)' in fs and b'path_str.strip_prefix("file:///").unwrap_or(&path_str)' in fs and b'let path = std::path::PathBuf::from(content.trim())' in fs and b'path.exists() && path.is_dir()' in fs
# Symbolic string identity only, not a filesystem operation.
assert 'file:///srv/project'[len('file:///'):]=='srv/project' and not 'srv/project/a b'.startswith('file:///srv/project') and 'file:///srv/project/a%20b'[len('file://'):]=='/srv/project/a%20b'
assert b'uri.strip_prefix("prism://project/")' in fs and b'percent_encoding::percent_decode_str(clean_uri).decode_utf8_lossy()' in fs and b'clean_uri.starts_with(&root_path)' in fs and b'let path = self.path.join(name)' in fs and b'load_file(uri).await' in fs and b'_uri.strip_prefix("file://").unwrap_or(_uri)' in fs
assert b'static FAILED_FETCHES:' in fs and b'guard.contains(&uri)' in fs and b'Fetch failed (cached)' in fs and b'if result.is_err() && uri.starts_with("blob:")' in fs and b'guard.insert(uri.clone())' in fs
assert b'std::fs::write(&self.path, data).map_err' in fs and b'std::fs::write(path, data).map_err' in fs and b'PROJECT_ROOT.lock().unwrap()' in fs and b'js_sys::Reflect::get(&next_obj, &"done".into())' in fs and b'Never use `unwrap` or `expect` outside tests' in rules
lockproof=load(a['historical_lock_membership_evidence']);shared=load(lockproof['shared_commit_and_tree_objects']);lnodes=dict(shared['tree_objects'])
for p in lockproof['additional_shared_tree_objects']:lnodes.update(load(p)['additional_tree_objects'])
lnodes.update(lockproof['additional_tree_objects'])
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as tar:oldlock=tar.extractfile(work['roles'][497]['historical_sha256']).read()
assert work['roles'][497]['status']=='lock-metadata-partial' and sha(oldlock)=='82b8a6ebb3685ec699069eed2cd8cdf76be94b6fe70f23b7e8eb2a62fe10c1ce';chain('Cargo.lock',lockproof['paths']['Cargo.lock'],oldlock,lockproof['root_tree'],lnodes)
packages=[('reqwest','0.12.28','eddd3ca559203180a307f12d114c268abf583f59b03cb906fd0b3ff8646c1147'),('percent-encoding','2.3.2','9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220'),('base64','0.22.1','72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6')]
for project,ver,checksum in packages:
 directory=root/f'native-evidence/{project}-{ver}';m=json.loads((directory/'MANIFEST.json').read_text());raw=(directory/m['archive']).read_bytes();assert sha(raw)==checksum==m['sha256'] and len(raw)==m['bytes'] and checksum.encode() in lock and checksum.encode() in oldlock
 with tarfile.open(directory/m['archive'],'r:gz') as tar:
  for x in m['inspected_members']:
   d=tar.extractfile(x['path']).read();assert len(d)==x['bytes'] and sha(d)==x['sha256']
   if x['path'].endswith('reqwest-0.12.28/src/lib.rs'):assert b'Client::builder().build()?.get(url).send().await' in d
   if x['path'].endswith('reqwest-0.12.28/src/async_impl/response.rs'):assert b'BodyExt::collect(self.res.into_body())' in d and b'pub fn error_for_status' in d and b'status.is_client_error() || status.is_server_error()' in d
   if x['path'].endswith('reqwest-0.12.28/src/async_impl/client.rs'):assert b'Poll::Ready(Ok(res))' in d and b'let res = Response::new(' in d
   if x['path'].endswith('percent-encoding-2.3.2/src/lib.rs'):assert b'pub fn percent_decode_str' in d and b'pub fn decode_utf8_lossy' in d
   if x['path'].endswith('base64-0.22.1/src/decode.rs'):assert b'pub enum DecodeError' in d and b'InvalidByte(usize, u8)' in d
  assert {f'{project}-{ver}/LICENSE-APACHE',f'{project}-{ver}/LICENSE-MIT'}<=set(tar.getnames())
  if project=='reqwest':
   qualification=load('native-evidence/reqwest-0.12.28/SCAN-QUALIFICATION.json');tls=tar.extractfile(qualification['member']).read();assert qualification['archive_sha256']==checksum and sha(tls)==qualification['member_sha256'];start=b'-----BEGIN RSA '+b'PRIVATE KEY-----';end=b'-----END RSA '+b'PRIVATE KEY-----';assert tls.count(start)==1;at=tls.index(start)+len(start);finish=tls.index(end,at);assert tls[at:finish]==b'\\n\\\n            '
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==21
adler=json.loads(records['adler/READ-L11-359.json']);assert adler['accepted_unique_count_change']==0
for i,scope in ((2259,[[1,53]]),(2260,[[635,705],[782,884]])):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==scope and x['sha256']==roles[i-2259]['sha256']
path='segovb/READ-L12-744.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-744.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256']
for i,scope in ((2259,[[1,53]]),(2260,[[635,705],[782,884]])):
 x=next(r for r in data['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==scope and x['source_sha256']==roles[i-2259]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
tail=data['additional_connected_body_receipts'][0];assert tail['path'] in records and sha(records[tail['path']])==tail['sha256'];d=json.loads(records[tail['path']]);assert [x['fresh_personal_body_reads'] for x in d['rows']]==[[[1,38],[54,128],[141,188],[210,227],[270,297],[314,349]],[[596,604]]] and all(x['new_original_credit']==0 and x['registration'] in records for x in d['rows'])
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (154,311,312,313,1829):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified partial filesystem attributions, native sources and conditional locator contracts; no product execution.')
