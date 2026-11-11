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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-FBX-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2251-2258.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2251,2259))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in range(2251,2259)]==[18,73,374,24,119,95,548,46]
# The earlier f82 native source paths are independent of historical6533 and
# current95e; verify the old commit, raw trees and each original object.
oldproof=load(a['older_native_git_membership_evidence']);f82=oldproof['revision'];assert f82=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';base=load(oldproof['shared_commit_and_tree_objects']);nodes82={x['git_object']:x for x in base['objects'] if x['type']=='tree'};ci=next(x for x in base['objects'] if x['type']=='commit');commit=contents(ci['preserved_bytes']);assert oid('commit',commit)==f82 and sha(commit)==ci['sha256'] and len(commit)==ci['bytes'] and commit.splitlines()[0]==b'tree '+oldproof['root_tree'].encode()
for p in oldproof['additional_shared_tree_objects']:nodes82.update(load(p)['additional_tree_objects'])
nodes82.update(oldproof['additional_tree_objects']);oldnative={}
for p,x in oldproof['paths'].items():
 raw=contents(x['preserved_bytes']);chain(p,x,raw,oldproof['root_tree'],nodes82);oldnative[sha(raw)]=raw
assert set(oldnative)=={'8dd8a3259e3f016b189332efb66b735c1473c70ce7e1c152da33a3e99250741a','abe7e01b2939a26e273d91fd975056216b87c87cf494a1869248762145469fd1','169a5b973235cf21f0d2c1c088acf3b6526d62758c83e14a0b1e624afcd648ed'}
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
animation=bodies[2253];encoding=bodies[2254];testsource=bodies[2255];fixture=bodies[2256];fbx=bodies[2257];names=bodies[2258];character=sources['crates/fabelgeist-mhr/src/character.rs'];skeleton=sources['crates/fabelgeist-mhr/src/character/skeleton.rs'];rig=sources['crates/fabelgeist-rig/src/lib.rs'];rules=sources['crates/AGENTS.md'];lock=sources['Cargo.lock']
assert b'while reader.pos + reader.header_size() <= data.len()' in fbx and b'let end_offset = reader.header_word()? as usize' in fbx and b'if end_offset == 0' in fbx and b'while reader.pos + sentinel <= end_offset' in fbx and b'reader.pos = end_offset' in fbx and b'Ok(Some(Node {' in fbx
# Symbolic offsets only: starting 27 plus an old-format 13-byte header
# leaves 40; resetting to 27 keeps the outer header condition true.
assert 27+13==40 and 40+13>27 and 27+13<=40
assert b'Vec::with_capacity(count)' in fbx and b'Vec::with_capacity(count * width)' in fbx and b'.read_to_end(&mut out)' in fbx and b'if bytes.len() < count * width' in fbx
assert b'FbxArrayEncoding::try_from(reader.u32()?)?' in fbx and b'Uncompressed = 0' in encoding and b'Zlib = 1' in encoding and b'pub reported_code: u32' in encoding and b'constructor = UnsupportedFbxArrayEncoding::from' in encoding
assert b'property.to_string()' in animation and b'"Lcl Translation" => entry.translation' in animation and b'"Lcl Rotation" => entry.rotation' in animation and b'"Lcl Scaling" => entry.scale' in animation
assert b'pub times: Vec<f64>' in animation and b'pub values: Vec<f64>' in animation and b'let count = times.len().min(values.len())' in animation and b'values[..count].to_vec()' in animation and b'self.times.partition_point' in animation and b'self.values[0]' in animation
assert b'pub objects: Vec<Object>' in fbx and b'by_id.insert(id, objects.len())' in fbx and b'self.by_id.get(&id).map(|i| &self.objects[*i])' in fbx and b'links.entry(to).or_default().push(Link { from, property })' in fbx and b'for root in scene.children(0)' in skeleton and b'for child in self.scene.children(object.id)' in skeleton and b'self.visit(child, Some(index))' in skeleton and b'let scene = Scene::parse(data)?' in character
assert b'.try_into().unwrap()' in fbx and b'nodes.last_mut().expect("just pushed")' in animation and b'Never use `unwrap` or `expect` outside tests' in rules
assert b'raw.windows(2).position(|word| word == [0, 1])' in names and b'qualified.0.rfind(\':\')' in names and b'impl From<&FbxObjectName> for RigJointName' in names and b'pub struct RigJointName' in rig
assert b'include_str!("fixtures/array_encoding.txt")' in testsource and len(fixture.splitlines())==95
# Verify the old 6533 lock through the already preserved Git path proof; do
# not change the partial original-role 497 status or imply a full read.
lockproof=load(a['historical_lock_membership_evidence']);lshared=load(lockproof['shared_commit_and_tree_objects']);lnodes=dict(lshared['tree_objects'])
for p in lockproof['additional_shared_tree_objects']:lnodes.update(load(p)['additional_tree_objects'])
lnodes.update(lockproof['additional_tree_objects'])
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as tar:oldlock=tar.extractfile(work['roles'][497]['historical_sha256']).read()
assert work['roles'][497]['status']=='lock-metadata-partial' and sha(oldlock)=='82b8a6ebb3685ec699069eed2cd8cdf76be94b6fe70f23b7e8eb2a62fe10c1ce';chain('Cargo.lock',lockproof['paths']['Cargo.lock'],oldlock,lockproof['root_tree'],lnodes)
packages=[('num_enum','0.7.6','5d0bca838442ec211fa11de3a8b0e0e8f3a4522575b5c4c06ed722e005036f26'),('num_enum_derive','0.7.6','680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8'),('flate2','1.1.10','6e634e2e0ebac1ee034020da1ca582e17ffe4e0f5e985823721e168928136dcb')];native={}
for project,ver,checksum in packages:
 directory=root/f'native-evidence/{project}-{ver}';m=json.loads((directory/'MANIFEST.json').read_text());raw=(directory/m['archive']).read_bytes();assert sha(raw)==checksum==m['sha256'] and len(raw)==m['bytes'] and checksum.encode() in lock and checksum.encode() in oldlock
 with tarfile.open(directory/m['archive'],'r:gz') as tar:
  for x in m['inspected_members']:
   d=tar.extractfile(x['path']).read();assert len(d)==x['bytes'] and sha(d)==x['sha256'];native[sha(d)]=d
   if x['path'].endswith('num_enum_derive-0.7.6/src/lib.rs'):assert b'#error_constructor ( number )' in d and b'type Error = #error_type' in d
   if x['path'].endswith('flate2-1.1.10/src/zlib/read.rs'):assert b'impl<R: Read> Read for ZlibDecoder<R>' in d and b'self.inner.read(into)' in d
  assert {f'{project}-{ver}/LICENSE-APACHE',f'{project}-{ver}/LICENSE-MIT'}<=set(tar.getnames())
for h,d in oldnative.items():assert h in native and d==native[h]
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==51
adler=json.loads(records['adler/READ-L11-358.json']);assert adler['accepted_unique_count_change']==0
for i in range(2251,2259):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==[[1,roles[i-2251]['lines']]] and x['sha256']==roles[i-2251]['sha256']
for batch,indices in ((743,range(2251,2258)),(744,[2258])):
 path=f'segovb/READ-L12-{batch}.json';d=json.loads(records[path]);v=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert d['root_personal_rereads']==[] and d['delegated_attribution_preserved'] and d['accepted_unique_count_change']==0 and v['receipt_sha256']==lookup[path]['sha256'] and d['proposal'] in records
 for i in indices:
  x=next(r for r in d['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,roles[i-2251]['lines']]] and x['source_sha256']==roles[i-2251]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
 for reg in d['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
 for x in d.get('additional_connected_body_receipts',[]):assert x['path'] in records and sha(records[x['path']])==x['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (152,153,303,304,305,306,307,308,309,1828,2942):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
family=a['connected_historical_reason_family'];row=next(x for x in ledger if x['id']==family['ledger_id']);assert [x['queue'] for x in row['named_subcontracts']]==family['queue_indices'] and all(x['id']==old[x['queue']]['contract_id'] for x in row['named_subcontracts']) and row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified FBX historical/native provenance and conditional parser, curve and graph source contracts; no product execution.')
