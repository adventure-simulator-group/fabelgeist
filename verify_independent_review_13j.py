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
 assert depth<=9 and sha(body)==m['published_sha256'] and len(body)==m['published_bytes'],where
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

for name in ('13J-HAIR-OPS',):
 d=load('INDEPENDENT-REVIEW-'+name+'.json');raw=(root/d['archive']).read_bytes();assert sha(raw)==d['archive_sha256'] and len(raw)==d['archive_bytes'];members={x['path']:x for x in d['members']}
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  files=t.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts and (x.uid,x.gid,x.uname,x.gname)==(0,0,'','') for x in files) and set(x.name for x in files)==set(members);data={x.name:t.extractfile(x).read() for x in files}
 for path,body in data.items():verify(body,members[path],name+'/'+path)
 assert 'exact changed top-level originals remain local only' in d['qualification']
 if name=='13I-RIG-IDENTITY':
  for line in data['SHA256SUMS'].decode().splitlines():
   h,p=line.split('  ',1);assert members[p]['original_sha256']==h
  assert members['staged-assessment.json']['original_sha256']=='0100b68d44a3d5bc579cc1ba7b7fd195f2fcba24cfaa7c316a7cd2d9954c2350' and members['staged-baseline.json.gz']['original_uncompressed_sha256']=='e6f91d9b03f9f789e8845d733322820a4fa2ab1094066688b82752e4b2f3b3d4'
 else:
  review=json.loads(data['REVIEW.json']);assert review['draft']['sha256']==members['reviewed-draft.py']['original_sha256']=='534c456d222de0e3ec1bc5acb317da2593dfca30c09869e58e923bfa73f63e5c'
print('Verified separately attributed independent review packets and qualified original/public variants; no product execution or reader transfer inferred.')
