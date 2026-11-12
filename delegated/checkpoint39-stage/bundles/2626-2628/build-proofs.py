import gzip,hashlib,json,pathlib,subprocess,tarfile
p=pathlib.Path(__file__).parent;repo='<PRIVATE_PATH_0001>';f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';pr='6533e444fa37552927070cc8d8190907fd3b413e';records=[]
def load(e):
 b=(p/e['archive']).read_bytes();return gzip.decompress(b) if e.get('encoding')=='gzip' else b
def add(kind,rev,path,archive,sha,encoding=None):records.append(dict(kind=kind,revision=rev,path=path,archive=archive,sha256=sha,encoding=encoding))
r=json.loads((p/'registration.json').read_bytes())
for e in r['entries']:
 if e['kind']=='native-package' and e.get('role'):
  add('original native recovered object via f82 transport',f82,'handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/'+e['sha256'],e['archive'],e['sha256'])
 elif e['kind']=='current-project':
  add('current pinned project',e['revision'],e['path'],e['archive'],e['sha256']);add('per-file 6533 equality; no historical personal read',pr,e['path'],e['archive'],e['sha256'])
 elif e['kind']=='project-lock':add('pinned lock',e['revision'],e['path'],e['archive'],e['sha256'],'gzip')
for e in json.loads((p/'historical-index.json').read_bytes())['records']:
 if e['kind']=='historical-774':add('historical 774 evidence via f82',f82,e['transport_path'],e['file'],e['sha256'],'gzip')
(p/'git-objects').mkdir(exist_ok=True);objs={};proof=[]
def data(t,oid):
 b=subprocess.check_output(['git','-C',repo,'cat-file',t,oid]);assert hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid
 if oid not in objs:
  dst=p/'git-objects'/(oid+'.'+t+'.gz');dst.write_bytes(gzip.compress(b,mtime=0));objs[oid]={'type':t,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'archive':str(dst.relative_to(p))}
 return b
def entries(b):
 d={};i=0
 while i<len(b):
  s=b.index(b' ',i);z=b.index(b'\0',s);d[b[s+1:z].decode()]=(b[i:s].decode(),b[z+1:z+21].hex());i=z+21
 return d
for x in records:
 commit=data('commit',x['revision']);tree=commit.splitlines()[0].split()[1].decode();root=tree;chain=[]
 for name in x['path'].split('/'):
  mode,child=entries(data('tree',tree))[name];chain.append({'tree':tree,'name':name,'mode':mode,'object':child});tree=child
 b=(p/x['archive']).read_bytes();b=gzip.decompress(b) if x.get('encoding')=='gzip' else b;oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert hashlib.sha256(b).hexdigest()==x['sha256'] and oid==tree;proof.append(dict(x,commit_tree=root,chain=chain,git_blob=oid))
(p/'git-proofs.json').write_text(json.dumps({'objects':objs,'proofs':proof,'qualification':'f82 transports native and historic evidence, not project revision. Per-file 6533 equality does not prove the recorded pr-working snapshot was clean or confer historical read credit.'},indent=2)+'\n');print('proofs',len(proof),'objects',len(objs))
