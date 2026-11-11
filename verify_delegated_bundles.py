#!/usr/bin/env python3
"""Verify delegated evidence, exact original provenance and archive safety only."""
import gzip,hashlib,io,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads((root/p).read_text())
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 d={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return d
checkpoint=load('DELEGATED-EVIDENCE-CHECKPOINT.json');assert checkpoint['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and checkpoint['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e'
transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']}
work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
byagent={};details={}
for bundle in checkpoint['bundles']:
 raw=(root/bundle['archive']).read_bytes();assert sha(raw)==bundle['archive_sha256'] and len(raw)==bundle['archive_bytes'];members={x['path']:x for x in bundle['members']};assert len(members)==len(bundle['members'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
  files=tar.getmembers();assert len(files)==len(members) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:tar.extractfile(x).read() for x in files}
 for path,body in data.items():
  x=members[path];assert sha(body)==x['published_sha256'] and len(body)==x['published_bytes']
  if x['redacted_nonessential_local_path_occurrences']:assert body.count(b'<LOCAL_PATH_')==x['redacted_nonessential_local_path_occurrences'] and 'reconstructed public copy' in x['qualification']
  else:assert sha(body)==x['original_sha256'] and len(body)==x['original_bytes']
 assert (root/bundle['report_path']).read_bytes()==data[bundle['report_archive_member']]
 byagent[bundle['agent']]=data;details[bundle['agent']]=members
 assert 'await central review' in bundle['publication_status']
assert len(byagent)==3
# Compare original 059 record copies to the independently preserved transport.
for agent,index,record_key,path_key,copy_key in [('/root/hair_ops','historical-manifest.json','records','original_transport_path','copy_path'),('/root/mhr_character','historical-059-index.json','records','path','local_exact_copy'),('/root/rig_identity','history-manifest.json','original_059_records','path_as_recorded','preserved_bytes')]:
 data=byagent[agent];record=json.loads(data[index]);rs=record[record_key]
 for x in rs:
  original=lookup[x[path_key]];raw=data[x[copy_key]];assert sha(raw)==x['sha256']==original['sha256'] and len(raw)==x['bytes']==original['bytes'] and oid('blob',raw)==x['git_blob']==original['git_blob']
assert len(json.loads(byagent['/root/hair_ops']['historical-manifest.json'])['records'])==27
assert len(json.loads(byagent['/root/mhr_character']['historical-059-index.json'])['records'])==17
assert len(json.loads(byagent['/root/rig_identity']['history-manifest.json'])['original_059_records'])==35
# Bind delegated role bodies to the current Git inventory and original source archive.
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 d=byagent['/root/hair_ops'];reg=json.loads(d['registration.json'])
 for x in reg['ordered_roles']:
  p=x['path'];body=d['source/'+Path(p).name];old=archive.extractfile(work[x['original_index']]['historical_sha256']).read();assert body==old and sha(body)==x['sha256'] and oid('blob',body)==x['git_blob']==tree[p]
 d=byagent['/root/mhr_character'];reg=json.loads(d['registration.json'])
 for x in reg['sources']:
  body=d[x['source_file']];assert sha(body)==x['sha256'] and oid('blob',body)==x['git_blob_sha1'] and len(body.splitlines())==x['lines']
  if x['kind']=='historical':assert body==archive.extractfile(work[x['original_role_index']]['historical_sha256']).read()
  else:assert tree[x['path']]==x['git_blob_sha1']
 d=byagent['/root/rig_identity'];reg=json.loads(d['registration.json'])
 for x in reg['entries']:
  body=d['sources/'+('95e6/' if x['revision'].startswith('95e6') else '6533/')+x['path']];assert sha(body)==x['sha256'] and oid('blob',body)==x['git_blob'] and len(body)==x['bytes']
  if x['revision'].startswith('95e6'):assert tree[x['path']]==x['git_blob']
  else:assert body==archive.extractfile(work[x['original_role']]['historical_sha256']).read()
# Verify older774 records against the older export's original digests, while
# keeping two normalized exports and publicly redacted original copies distinct.
mhr=byagent['/root/mhr_character'];index=json.loads(mhr['older-774/INDEX.json']);manifest=json.loads(mhr[index['export_manifest_file']]);assert sha(mhr[index['export_manifest_file']])==index['export_manifest_sha256']
for x in index['entries']:
 original=x['original_local_copy'];info=details['/root/mhr_character'][original];assert info['original_sha256']==x['original_sha256']==x['older_export_manifest_entry']['original_record_sha256'] and info['original_bytes']==x['original_bytes']
 if x['export_differs_from_original']:
  normal=mhr[x['normalized_export_local_copy']];assert sha(normal)==x['normalized_export_sha256']==x['older_export_manifest_entry']['published_export_sha256'] and x['normalized_export_sha256']!=x['original_sha256']
# Exact f82 tree membership for the two recovered older supplier/producer records.
proof=load('DELEGATED-OLDER-RIG-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={x['git_object']:x for x in shared['objects'] if x['type']=='tree'}
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);rig=byagent['/root/rig_identity']
for path,x in proof['paths'].items():
 current=proof['root_tree'];assert [s['name'] for s in x['chain']]==path.split('/')
 for step in x['chain']:
  node=nodes[current];raw=gzip.decompress((root/node['preserved_bytes']).read_bytes());assert sha(raw)==node['sha256'] and len(raw)==node['bytes'] and oid('tree',raw)==current==step['tree'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 body=rig[x['bundle_member']];assert current==x['git_blob']==oid('blob',body) and sha(body)==x['sha256'] and len(body)==x['bytes']
assert len(proof['paths'])==2 and proof['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_evidence_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT.json'
print('Verified 200 delegated evidence members, historical hashes and older rig Git paths; central reconciliation remains open; no product execution.')
