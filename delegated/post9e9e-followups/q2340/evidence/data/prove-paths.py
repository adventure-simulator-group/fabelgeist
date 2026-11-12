import json,hashlib,gzip
from pathlib import Path
p=Path(__file__).resolve().parent;reg=json.loads((p/'PROOF-REGISTRATION.json').read_text());extras=json.loads((p/'PROOF-EXTRA-REGISTRATION.json').read_text())['extra'];old=Path(reg['prior_trees']);outdir=p/'proof'/'git-trees';outdir.mkdir(parents=True,exist_ok=False)
def oid(kind,data):return hashlib.sha1(kind.encode()+b" "+str(len(data)).encode()+b"\0"+data).hexdigest()
def tree_data(id):
 f=old/(id+'.tree')
 if f.exists():raw=f.read_bytes()
 else:
  f=Path(extras[id]);raw=f.read_bytes();raw=gzip.decompress(raw) if f.suffix=='.gz' else raw
 assert oid('tree',raw)==id
 dst=outdir/(id+'.tree')
 if not dst.exists():
  with dst.open('xb') as w:w.write(raw)
 return raw
def parse_tree(data):
 out={};i=0
 while i<len(data):
  sp=data.index(b' ',i);z=data.index(b'\0',sp);mode=data[i:sp].decode();name=data[sp+1:z].decode();key=data[z+1:z+21].hex();out[name]=(mode,key);i=z+21
 return out
results=[]
for label,paths in [('95e',reg['current_paths']),('6533',reg['frozen_paths'])]:
 commit=Path(reg['commits'][label]).read_bytes();commit_oid=oid('commit',commit);expected={'95e':'95e63628f9d70b8dd6e6fede64334696c6995009','6533':'6533e444fa37552927070cc8d8190907fd3b413e'}[label];assert commit_oid==expected;root=commit.split(b'\n',1)[0].split()[1].decode()
 for path in paths:
  cur=root;chain=[]
  for comp in path.split('/'):
   mode,nextid=parse_tree(tree_data(cur))[comp];chain.append({'tree':cur,'component':comp,'mode':mode,'oid':nextid});cur=nextid
  roots=[p/'sources'/'95e',p/'sources'/'95e63628f9d70b8dd6e6fede64334696c6995009'];f=next(r/path for r in roots if (r/path).exists());raw=f.read_bytes();blob=oid('blob',raw);results.append({'revision':label,'commit':commit_oid,'path':path,'chain':chain,'source_sha256':hashlib.sha256(raw).hexdigest(),'source_bytes':len(raw),'computed_blob':blob,'expected_blob':cur,'match':blob==cur})
assert all(x['match'] for x in results)
with (p/'PROOF-RESULT.json').open('x') as f:json.dump({'paths':results,'tree_count':len(list(outdir.iterdir())),'qualification':'Git object reachability proves per-path bytes only, not whole-tree or semantic coverage equivalence'},f,indent=2)
print('verified paths',len(results),'trees',len(list(outdir.iterdir())))
for x in results:print(x['revision'],x['path'],x['computed_blob'])
