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
idx=load('CENTRAL-13B-WINDOWS-PROOF.json');raw=(R/'delegated/central-13b-windows-proof.tar.gz').read_bytes();assert sha(raw)==idx['sha256'] and len(raw)==idx['bytes']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
 files=tar.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:tar.extractfile(x).read() for x in files};assert len(data)==len(files)
p=json.loads(data['proof.json']);objects={}
for key,x in p['objects'].items():
 body=data[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==key;objects[key]=body
assert set(data)=={'proof.json'}|{x['archive'] for x in p['objects'].values()};reg=load('CENTRAL-13B-REGISTRATION.json');assert len(p['paths'])==idx['paths']==len(reg['rows'])==6 and len(objects)==idx['objects']==14
for x,row in zip(p['paths'],reg['rows']):
 assert (x['path'],x['revision'],x['blob'],x['sha256'])==(row['path'],row['revision'],row['git_blob'],row['sha256'])
 assert objects[x['revision']].splitlines()[0]==b'tree '+x['root_tree'].encode();cur=x['root_tree'];names=[]
 for s in x['chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'],s['object']);cur=s['object'];names.append(s['name'])
 assert names==x['path'].split('/') and cur==x['blob'] and sha(objects[cur])==x['sha256'] and len(objects[cur])==x['bytes'] and len(objects[cur].splitlines())==row['actual_eof']
 for lo,hi in row['planned_display_ranges']:assert 1<=lo<=hi<=row['actual_eof']
reg_bytes=(R/'CENTRAL-13B-REGISTRATION.json').read_bytes();receipt=load('CENTRAL-13B-READ-RECEIPT.json');assert sha(reg_bytes)==receipt['registration_sha256'] and len(receipt['reads'])==len(reg['rows'])
for a,b in zip(reg['rows'],receipt['reads']):
 assert all(a[k]==b[k] for k in ('revision','path','git_blob','sha256')) and a['planned_display_ranges']==b['actual_display_ranges']
print('Verified six pinned current source blobs and tree paths; only receipt windows count as personal display.')

for suffix in ('SUPPORT','SUPPORT2'):
 data=(R/('CENTRAL-13B-'+suffix+'-REGISTRATION.json')).read_bytes();reg2=json.loads(data);rec=load('CENTRAL-13B-'+suffix+'-READ-RECEIPT.json');assert sha(data)==rec['registration_sha256'] and len(reg2['rows'])==len(rec['reads'])==1
 x=reg2['rows'][0];y=rec['reads'][0];assert all(x[k]==y[k] for k in ('revision','path','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges'];assert any((z['revision'],z['path'],z['blob'],z['sha256'])==(x['revision'],x['path'],x['git_blob'],x['sha256']) for z in p['paths']);assert all(1<=a<=b<=x['actual_eof'] for a,b in x['planned_display_ranges'])
print('Verified two supplementary registration and receipt pairs against the same proved source blob.')
