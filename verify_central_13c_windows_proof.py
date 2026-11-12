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
idx=load('CENTRAL-13C-WINDOWS-PROOF.json');raw=(R/'delegated/central-13c-windows-proof.tar.gz').read_bytes();assert sha(raw)==idx['sha256'] and len(raw)==idx['bytes']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
 files=tar.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:tar.extractfile(x).read() for x in files};assert len(data)==len(files)
p=json.loads(data['proof.json']);objects={}
for key,x in p['objects'].items():
 body=data[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==key;objects[key]=body
assert set(data)=={'proof.json'}|{x['archive'] for x in p['objects'].values()};reg=load('CENTRAL-13C-REGISTRATION.json');assert len(p['paths'])==idx['paths']==len(reg['rows'])==9 and len(objects)==idx['objects']==30
for x,row in zip(p['paths'],reg['rows']):
 assert (x['path'],x['revision'],x['blob'],x['sha256'])==(row['path'],row['revision'],row['git_blob'],row['sha256'])
 assert objects[x['revision']].splitlines()[0]==b'tree '+x['root_tree'].encode();cur=x['root_tree'];names=[]
 for s in x['chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'],s['object']);cur=s['object'];names.append(s['name'])
 assert names==x['path'].split('/') and cur==x['blob'] and sha(objects[cur])==x['sha256'] and len(objects[cur])==x['bytes'] and len(objects[cur].splitlines())==row['actual_eof']
 for lo,hi in row['planned_display_ranges']:assert 1<=lo<=hi<=row['actual_eof']
reg_bytes=(R/'CENTRAL-13C-REGISTRATION.json').read_bytes();receipt=load('CENTRAL-13C-READ-RECEIPT.json');assert sha(reg_bytes)==receipt['registration_sha256'] and len(receipt['reads'])==len(reg['rows'])
for a,b in zip(reg['rows'],receipt['reads']):
 assert all(a[k]==b[k] for k in ('revision','path','git_blob','sha256')) and a['planned_display_ranges']==b['actual_display_ranges']
print('Verified nine current and historical-transport source blobs and tree paths; only receipt windows count as personal display.')

old,new=reg['rows'][:2];assert old['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee' and 'transport revision is not source revision' in old['reason'];assert new['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and old['sha256']!=new['sha256']
a,b=objects[old['git_blob']].splitlines(),objects[new['git_blob']].splitlines()
for lo,hi in old['planned_display_ranges']:assert a[lo-1:hi]==b[lo-1:hi]
print('Verified matching selected historical/current source windows despite distinct whole-file hashes; f82 remains transport only.')
