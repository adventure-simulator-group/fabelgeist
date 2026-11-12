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

reg=json.loads((ROOT/'registration.json').read_text());proofs=[]
for r in reg['sources']:
 if r['kind']=='assigned-followup':
  if r['identity'].startswith('Recovered'):
   path='handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/'+r['sha256']
   proofs.append(prove(TRANS,path,r['file'],qualification='f82 transport of recovered pr-working evidence; not project revision'))
   proofs.append(prove(OLD,r['path'],r['file'],qualification='Per-file 6533 equality, no separate 6533 personal display or whole-tree cleanliness'))
  else:proofs.append(prove(NEW,r['path'],r['file'],qualification='Separately displayed pinned current source'))
 else:proofs.append(prove(NEW,r['path'],r['file'],qualification='Prospectively registered bounded current support, no ordered-role credit'))
ext=json.loads((ROOT/'support-extension-registration.json').read_text())
for r in ext['sources']:proofs.append(prove(NEW,r['path'],r['file'],qualification='Fresh bounded current support, no transferred attribution'))
for fn,path in [('findings.jsonl.gz','rig/admission03/findings.jsonl'),('source-range-registration.jsonl.gz','rig/admission03/source-range-registration.jsonl'),('additional-contracts.jsonl.gz','rig/admission03/additional-contracts.jsonl'),('contracts-012.json.gz','rig/initial06-type-ownership/contracts-012.json')]:
 proofs.append(prove(TRANS,'handoff/774/adler-original/audit/continuation/'+path,'historical/history-774/'+fn,'gzip','Historical 774 rig original, fresh display limited to registered records'))
proofs.append(prove('98cc9fcb9373f5deaa39703f6ff4353ad0b4ab07','baseline-059/queue.json','historical/baseline-059-queue-committed-2e1f.json.gz','gzip','Older 059, not checkpoint3821'))
proofs.append(prove('d7b14a15ef132422eca86e7d898b47423af92779','baseline-059/queue.json','historical/baseline-059-queue-committed-d7.json.gz','gzip','Later 059, same at local a0d; not checkpoint3821'))
proofs.append(prove(reg['committed_revision'],'BASELINE-95E-20261110.json','reference/committed-baseline.json.gz','gzip','Local committed 95e baseline capture, not a live remote verification'))
proofs.append(prove(reg['committed_revision'],'recovered/original-registry-source-archive/registry.json.gz','historical/original-registry.json.gz','none','Stored original registry gzip identity'))
result={'schema':'726-minimal-git-source-proofs-v1','objects':objects,'proofs':proofs,'qualification':'Only source, transport and local committed-evidence membership; raw commit metadata may identify authors and needs privacy review. Working snapshot is uncommitted observation only. No product execution.'}
p=ROOT/'git-proofs.json'
with p.open('x') as f:f.write(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'objects':len(objects),'proofs':len(proofs),'commits':sum(x['type']=='commit' for x in objects.values())}))
