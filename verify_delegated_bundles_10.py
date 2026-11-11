#!/usr/bin/env python3
"""Verify complete delegated packet bytes and privacy; no product checks."""
import gzip,hashlib,io,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);out[n.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
def verify(body,m,where,depth=0):
 assert depth<=5 and sha(body)==m['published_sha256'] and len(body)==m['published_bytes'],where
 encoding=m['content_encoding'];assert encoding in ('identity','gzip','tar','gzip-tar');plain=gzip.decompress(body) if encoding.startswith('gzip') else body
 assert sha(plain)==m['published_uncompressed_sha256'] and len(plain)==m['published_uncompressed_bytes'] and m['changed']==(m['published_sha256']!=m['original_sha256'])
 if not m['changed']:assert len(body)==m['original_bytes'] and sha(plain)==m['original_uncompressed_sha256'] and len(plain)==m['original_uncompressed_bytes'] and m['total_local_path_redactions']==0
 else:assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions']>0
 if encoding.endswith('tar'):
  rows={x['path']:x for x in m['members']};assert len(rows)==len(m['members']) and m['direct_local_path_redactions']==0
  with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
   files=t.getmembers();assert len(files)==len(rows) and all((x.isfile() or x.isdir()) and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);assert set(x.name for x in files)==set(rows)
   for x in files:
    assert not local.search(x.name.encode()) and not secret.search(x.name.encode());assert all(not local.search(v.encode()) and not secret.search(v.encode()) for v in (x.uname,x.gname));assert all(not local.search(str(v).encode()) and not secret.search(str(v).encode()) for v in x.pax_headers.values())
    if x.isdir():assert rows[x.name]['entry_type']=='directory' and not rows[x.name]['changed'] and rows[x.name]['total_local_path_redactions']==0
    else:verify(t.extractfile(x).read(),rows[x.name],where+'!'+x.name,depth+1)
  assert m['total_local_path_redactions']==sum(x['total_local_path_redactions'] for x in rows.values())
 else:assert not local.search(plain) and not secret.search(plain) and plain.count(b'<LOCAL_PATH_')>=m['direct_local_path_redactions'] and m['total_local_path_redactions']==m['direct_local_path_redactions'],where
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json')
assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and len(cp['bundles'])==27
work=load('COVERAGE-L01-ROLES.json')['roles'];historical=set();total=0;redactions=0
manifest_names=('MANIFEST.json','BUNDLE-MANIFEST.json','packet-manifest.json','manifest.json')
native_expected={('sdk-noiz-sats-hair','archives/noiz-0.5.0.crate'):'937ac4a1ea826fc4977c4db2629fa1278e998012c24e7f9c2be32020c66d7815',('sdk-noiz-sats-hair','archives/spacetimedb-sats-2.6.1.crate'):'6cefb87b7d5e760d43cb65855591e25af772bdcca19562e5796ae797240d1998',('p16-animation','source/glam-0.32.1.crate'):'f70749695b063ecbf6b62949ccccde2e733ec3ecbbd71d467dca4e5c6c97cca0',('p16-shutters-mesh','native-historical-packages/bevy_mikktspace-1.0.0.crate'):'bff34eb29ff4b8a8688bc7299f14fb6b597461ca80fec03ed7d22939ab33e48f',('p16-shutters-mesh','native-historical-packages/bevy_mesh-0.19.1.crate'):'e2adbf8e5a5817c345986756eb0377e18a510e1d7685ef62d6c7bafcd3bab85e'}
seen=set();snapshot_ids={}
def flatten(m):
 yield m
 for child in m.get('members',[]):yield from flatten(child)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for b in cp['bundles']:
  name=b['bundle_id'];assert name not in seen;seen.add(name);raw=(root/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes'];members={m['path']:m for m in b['members']};assert len(members)==len(b['members']);total+=len(members);redactions+=sum(m['total_local_path_redactions'] for m in members.values())
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
   files=t.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts and (x.uid,x.gid,x.uname,x.gname)==(0,0,'','') for x in files) and {x.name for x in files}==set(members)
   data={x.name:t.extractfile(x).read() for x in files}
  for path,body in data.items():verify(body,members[path],name+'/'+path)
  assert data[b['report_archive_member']]==(root/b['report_path']).read_bytes()
  manifests=[n for n in manifest_names if n in data];assert len(manifests)==1,(name,manifests)
  manifest=json.loads(data[manifests[0]]);rows=manifest['files'];assert len(rows)>0
  for row in rows:
   m=members[row['path']];assert row['sha256']==m['original_sha256'] and row['bytes']==m['original_bytes']
  for path in set(members)-{x['path'] for x in rows}-{manifests[0]}:
   assert path in ('prior-facing/MANIFEST.json','prior-slip/MANIFEST.json','prior-physics-contact/MANIFEST.json'),(name,path)
  content_by_sha={m['original_sha256']:m for top in members.values() for m in flatten(top) if 'original_sha256' in m}
  for i in b['scope_original_roles']:
   assert i not in historical;historical.add(i);r=work[i];h=r['historical_sha256'];assert h in content_by_sha,(name,i)
   body=archive.extractfile(h).read();assert sha(body)==h and len(body)==content_by_sha[h]['original_bytes']
   snapshot_ids[i]=r['snapshot']
  for key,expected in native_expected.items():
   if key[0]==name:
    member=members[key[1]];assert member['original_sha256']==expected and member['content_encoding']=='gzip-tar'
    # Exact package bytes are retained when unchanged; otherwise their public
    # reconstruction is explicitly qualified and the original checksum remains.
    if member['changed']:assert 'Reconstructed public copy' in member['qualification']
    else:assert sha(data[key[1]])==expected
assert len(historical)==len(set(historical)) and all(snapshot_ids[i]=='report-p17' for i in range(2460,2468))
assert snapshot_ids[2476]=='sdk-noiz-0.5.0' and snapshot_ids[2477]=='sdk-spacetimedb-sats-2.6.1'
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch10_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-10.json'
print('Verified',len(cp['bundles']),'complete delegated packets,',total,'members,',redactions,'recursive path redactions and',len(historical),'registered original role byte identities; assessments remain pending.')
