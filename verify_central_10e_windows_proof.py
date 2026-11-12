#!/usr/bin/env python3
"""Verify pinned source membership for the coordinator's bounded displays."""
import hashlib,io,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();oid=lambda k,b:hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,name=b[i:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
idx=load('CENTRAL-10E-CURRENT-WINDOWS-PROOF.json');raw=(R/'delegated/central-10e-current-windows-proof.tar.gz').read_bytes();assert sha(raw)==idx['sha256'] and len(raw)==idx['bytes']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
 files=tar.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:tar.extractfile(x).read() for x in files};assert len(data)==len(files)
p=json.loads(data['proof.json']);objects={}
for key,x in p['objects'].items():
 body=data[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==key;objects[key]=body
assert set(data)=={'proof.json'}|{x['archive'] for x in p['objects'].values()};reg=load('CENTRAL-10E-REGISTRATION.json');assert len(p['paths'])==idx['paths']==len(reg['rows'])==6 and len(objects)==idx['objects']==14
for x,row in zip(p['paths'],reg['rows']):
 assert (x['path'],x['revision'],x['blob'],x['sha256'])==(row['path'],row['revision'],row['git_blob'],row['sha256'])
 assert objects[x['revision']].splitlines()[0]==b'tree '+x['root_tree'].encode();cur=x['root_tree'];names=[]
 for s in x['chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'],s['object']);cur=s['object'];names.append(s['name'])
 assert names==x['path'].split('/') and cur==x['blob'] and sha(objects[cur])==x['sha256'] and len(objects[cur])==x['bytes'] and len(objects[cur].splitlines())==row['actual_eof']
 for lo,hi in row['planned_display_ranges']:assert 1<=lo<=hi<=row['actual_eof']
print('Verified six pinned current source blobs and tree paths; only receipt windows count as personal display.')
