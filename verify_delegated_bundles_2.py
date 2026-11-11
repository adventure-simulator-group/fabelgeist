#!/usr/bin/env python3
"""Verify second delegated evidence checkpoint and native source only."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 d={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return d
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-2.json');assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cp['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e'
transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 h,p=line.split('\t',1);tree[p]=h.split()[2]
by={};meta={}
for bundle in cp['bundles']:
 raw=(root/bundle['archive']).read_bytes();assert sha(raw)==bundle['archive_sha256'] and len(raw)==bundle['archive_bytes'];members={x['path']:x for x in bundle['members']};assert len(members)==len(bundle['members'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  files=t.getmembers();assert len(files)==len(members) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files}
 for p,b in data.items():
  x=members[p];assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes']
  if x['redacted_nonessential_local_path_occurrences']:assert b.count(b'<LOCAL_PATH_')==x['redacted_nonessential_local_path_occurrences'] and 'reconstructed public copy' in x['qualification']
  else:assert sha(b)==x['original_sha256'] and len(b)==x['original_bytes']
 assert data[bundle['report_archive_member']]==(root/bundle['report_path']).read_bytes();by[bundle['agent']]=data;meta[bundle['agent']]=members
assert len(by)==3 and sum(len(b['members']) for b in cp['bundles'])==247
for agent,index,key,pathkey,copykey in [('/root/mhr_character','historical-059-index.json','records','path','copy'),('/root/rig_identity','history-manifest.json','original_059_records','original_path','preserved_bytes'),('/root/hair_ops','historical-manifest.json','records','original_transport_path','copy_path')]:
 d=by[agent];items=json.loads(d[index])[key]
 for x in items:
  original=lookup[x[pathkey]];raw=d[x[copykey]];assert sha(raw)==x['sha256']==original['sha256'] and len(raw)==x['bytes']==original['bytes'] and oid('blob',raw)==x['git_blob']==original['git_blob']
assert [len(json.loads(by[a][i])[k]) for a,i,k in [('/root/mhr_character','historical-059-index.json','records'),('/root/rig_identity','history-manifest.json','original_059_records'),('/root/hair_ops','historical-manifest.json','records')]]==[25,23,20]
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 d=by['/root/mhr_character'];reg=json.loads(d['registration.json'])
 for x in reg['sources']:
  body=d[x['source_file']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
  if x['kind']=='historical':assert body==ar.extractfile(work[x['original_role_index']]['historical_sha256']).read()
  else:assert tree[x['path']]==x['git_blob_sha1']
 d=by['/root/rig_identity'];reg=json.loads(d['registration.json'])
 for x in reg['entries']:
  body=d['sources/'+('95e6/' if x['revision'].startswith('95e6') else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
  if x['revision'].startswith('95e6'):assert tree[x['path']]==x['git_blob']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
 d=by['/root/hair_ops'];reg=json.loads(d['registration.json'])
 for x in reg['ordered_roles']:
  body=d['source/'+Path(x['path']).name];assert sha(body)==x['sha256'] and oid('blob',body)==x['git_blob']==tree[x['path']] and body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
# Pinned Rayon archive, exact source member and current lock; inspect source only.
d=by['/root/hair_ops'];native=json.loads(d['native-registration-rayon.json']);archive=d[native['archive']];assert sha(archive)==native['archive_sha256'] and len(archive)==native['archive_bytes'];lock=d['source/support-Cargo.lock'];assert oid('blob',lock)==tree['Cargo.lock'];pkg=next(x for x in tomllib.loads(lock.decode())['package'] if x['name']=='rayon' and x['version']=='1.12.0');assert pkg['checksum']==sha(archive)
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:source=t.extractfile(native['source_member']).read()
assert source==d['source/rayon-slice-mod.rs'] and sha(source)==native['source_member_sha256'] and b'assert!(chunk_size != 0, "chunk_size must not be zero")' in source
# Older rig-skin proof: original blob verified before redaction; the one
# redacted public variant retains original digest/length and a separate copy.
proof=load('DELEGATED-OLDER-RIG-SKIN-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={x['git_object']:x for x in shared['objects'] if x['type']=='tree'}
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);rig=by['/root/rig_identity'];rigmeta=meta['/root/rig_identity']
for path,x in proof['paths'].items():
 current=proof['root_tree'];assert [s['name'] for s in x['chain']]==path.split('/')
 for step in x['chain']:
  n=nodes[current];raw=gzip.decompress((root/n['preserved_bytes']).read_bytes());assert sha(raw)==n['sha256'] and len(raw)==n['bytes'] and oid('tree',raw)==current==step['tree'] and entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 info=rigmeta[x['bundle_member']];body=rig[x['bundle_member']];assert current==x['git_blob'] and x['sha256']==info['original_sha256'] and x['bytes']==info['original_bytes']
 if info['redacted_nonessential_local_path_occurrences']==0:assert oid('blob',body)==current and sha(body)==x['sha256']
 else:assert 'historical-774-variant/' in x['bundle_member']
assert len(proof['paths'])==5 and proof['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch2_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-2.json'
print('Verified 247 delegated members, 68 original059 records, older rig path proofs and pinned Rayon source; no product execution.')
