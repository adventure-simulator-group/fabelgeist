#!/usr/bin/env python3
"""Freeze minimal raw Git object chains for registered evidence; no product execution.
Run only while building this new packet. The frozen verifier is read-only.
"""
import gzip, hashlib, json, pathlib, subprocess
ROOT = pathlib.Path(__file__).resolve().parent
REPO = '<PRIVATE_PATH_0001>'
OLD = '6533e444fa37552927070cc8d8190907fd3b413e'
NEW = '95e63628f9d70b8dd6e6fede64334696c6995009'
TRANS = 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
objects = {}

def capture(oid, typ):
    if oid in objects:
        assert objects[oid]['type'] == typ
        return gzip.decompress((ROOT / objects[oid]['file']).read_bytes())
    raw = subprocess.check_output(['git', '-C', REPO, 'cat-file', typ, oid])
    assert hashlib.sha1((typ+' '+str(len(raw))+'\0').encode() + raw).hexdigest() == oid
    rel = 'git-objects/'+oid+'.'+typ+'.gz'
    (ROOT/'git-objects').mkdir(exist_ok=True)
    (ROOT/rel).write_bytes(gzip.compress(raw,mtime=0))
    objects[oid] = {'type':typ, 'bytes':len(raw), 'sha256':hashlib.sha256(raw).hexdigest(), 'file':rel}
    return raw

def entries(raw):
    out={}; i=0
    while i < len(raw):
        j=raw.index(b' ',i); k=raw.index(b'\0',j+1)
        mode=raw[i:j].decode('ascii'); name=raw[j+1:k].decode('utf-8'); oid=raw[k+1:k+21].hex()
        assert len(raw[k+1:k+21])==20 and name not in out
        out[name]=(mode,oid); i=k+21
    return out

def prove(rev, path, local, compression='none', qualification=''):
    commit = capture(rev, 'commit')
    tree = commit.split(b'\n',1)[0].decode().split(' ')[1]
    chain=[]
    for name in path.split('/'):
        mode, oid = entries(capture(tree,'tree'))[name]
        chain.append({'tree':tree,'name':name,'mode':mode,'object':oid})
        tree=oid
    raw=(ROOT/local).read_bytes()
    if compression=='gzip': raw=gzip.decompress(raw)
    blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert blob==tree and chain[-1]['mode'] in ('100644','100755')
    return {'revision':rev,'path':path,'local':local,'compression':compression,'blob':blob,'sha256':hashlib.sha256(raw).hexdigest(),'chain':chain,'qualification':qualification}

reg=json.loads((ROOT/'REGISTRATION.json').read_text());proofs=[]
for r in reg['sources']:proofs.append(prove(NEW,r['path'],r['file'],qualification='Fresh personal registered bounded source; source file full bytes archived for identity only'))
vec=json.loads((ROOT/'REGISTRATION-VEC3.json').read_text())['source'];proofs.append(prove(NEW,vec['path'],vec['file'],qualification='Separately registered bounded Vec3 support'))
for fn in ('contract-groups.jsonl.gz','findings.jsonl.gz','source-range-registration.jsonl.gz','additional-contracts.jsonl.gz'):
 proofs.append(prove(TRANS,'handoff/774/adler-original/audit/continuation/rig/admission03/'+fn.removesuffix('.gz'),'prior42/historical-774/'+fn,'gzip','774 original rig history; personal display limited to registered lines'))
x={'schema':'726-independent-13m-git-source-proofs-v1','objects':objects,'proofs':proofs,'qualification':'Pinned current and historical transport identities only; exact commit objects contain author metadata and need privacy review. No historical/current reader credit transfer.'};p=ROOT/'git-proofs.json'
with p.open('x') as f:f.write(json.dumps(x,indent=2)+'\n')
print(json.dumps({'objects':len(objects),'proofs':len(proofs),'commits':sum(x['type']=='commit' for x in objects.values())}))
