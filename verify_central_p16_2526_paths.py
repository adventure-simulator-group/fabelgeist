#!/usr/bin/env python3
"""Verify individual P16 clean-base and current Git paths, not dirty snapshot cleanliness."""
import hashlib, io, json, tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
sha=lambda b: hashlib.sha256(b).hexdigest()
def oid(kind,b): return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(body):
    out={}; pos=0
    while pos < len(body):
        end=body.index(b'\0',pos); mode,name=body[pos:end].split(b' ',1)
        assert end+21 <= len(body) and name.decode() not in out
        out[name.decode()]=(mode.decode(),body[end+1:end+21].hex()); pos=end+21
    assert pos==len(body)
    return out
idx=json.loads((R/'CENTRAL-P16-2526-2530-PATH-PROOFS.json').read_text())
raw=(R/'delegated/central-p16-2526-2530-path-proofs.tar.gz').read_bytes()
assert sha(raw)==idx['sha256'] and len(raw)==idx['bytes']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
    files=tar.getmembers(); assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files)
    data={x.name:tar.extractfile(x).read() for x in files}; assert len(data)==len(files)
p=json.loads(data['proof.json']); objs={}
for key,x in p['objects'].items():
    body=data[x['archive']]; assert len(body)==x['bytes'] and sha(body)==x['sha256'] and oid(x['type'],body)==key
    objs[key]=body
assert set(data)=={'proof.json'}|{x['archive'] for x in p['objects'].values()}
work=json.loads((R/'COVERAGE-L01-ROLES.json').read_text())['roles']
advance=json.loads((R/'MAIN-ADVANCE-95E.json').read_text()); current={}
for line in (R/advance['trees']['current']['path']).read_text().splitlines():
    h,path=line.split('\t',1); current[path]=h.split()[2]
old='a120390947c89b703dc74c4255a13c035d054c50'; rev='95e63628f9d70b8dd6e6fede64334696c6995009'
assert p['historical_revision']==old and p['working_revision']==rev
assert {(x['original_role'],x['revision']) for x in p['paths']}=={(i,r) for i in [2526,2527,2528,2529,2530] for r in (old,rev)}
for x in p['paths']:
    i=x['original_role'];path=x['path']; assert path==work[i]['source_id'].split(':',1)[1]
    assert objs[x['revision']].splitlines()[0]==b'tree '+x['root_tree'].encode()
    cur=x['root_tree']; consumed=[]
    for s in x['chain']:
        assert s['tree']==cur and entries(objs[cur])[s['name']]==(s['mode'],s['object'])
        consumed.append(s['name']);cur=s['object']
    if x['status']=='present':
        assert consumed==path.split('/') and cur==x['blob']
        body=objs[cur];assert oid('blob',body)==cur and sha(body)==x['sha256'] and len(body)==x['bytes']
        if x['revision']==old: assert x['sha256']==work[i]['historical_sha256']
        else: assert current[path]==cur
    else:
        assert x['revision']==rev and x['status']=='absent' and path not in current
        m=x['missing'];assert m['tree']==cur and m['name'] not in entries(objs[cur])
        assert consumed+[m['name']]+m['remaining']==path.split('/')
assert len(p['paths'])==idx['paths']==10 and len(objs)==idx['objects']==24
assert sum(x['status']=='present' for x in p['paths'])==idx['present']==10
assert sum(x['status']=='absent' for x in p['paths'])==idx['absent']==0
print('Verified 10 P16 per-file Git path proofs; membership is not a personal read or checkpoint credit.')
