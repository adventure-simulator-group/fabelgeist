import gzip,hashlib,json,pathlib,subprocess
p=pathlib.Path(__file__).parent;repo='<PRIVATE_PATH_0001>';f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';reg=json.loads((p/'registration.json').read_bytes());records=[]
for row in reg['rows']:
 for x in row['objects']:records.append({'kind':'project-role','revision':x['revision'],'path':x['git_path'],'archive':x['local_file'],'sha256':x['sha256'],'blob':x['git_blob']})
 x=row['objects'][0];records.append({'kind':'historical-object-transport','revision':f82,'path':'handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/'+x['sha256'],'archive':x['local_file'],'sha256':x['sha256'],'blob':x['git_blob']})
for file in ['support-registration.json','support-registration-02.json','support-registration-03.json','support-registration-04.json']:
 for x in json.loads((p/file).read_bytes())['rows']:records.append({'kind':'support','revision':x['revision'],'path':x['path'],'archive':x['file'],'sha256':x['sha256'],'blob':x['blob']})
x=json.loads((p/'identity-comparison.json').read_bytes());records.append({'kind':'historical-support-identity-only-no-read','revision':x['historical_revision'],'path':x['path'],'archive':x['archive'],'sha256':x['sha256'],'blob':x['git_blob']})
for x in json.loads((p/'historical-index.json').read_bytes())['records']:
 if x['kind']=='historical-774':records.append({'kind':'historical-774','revision':f82,'path':x['transport_path'],'archive':x['file'],'sha256':x['sha256'],'blob':x['git_blob'],'encoding':'gzip'})
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
 b=(p/x['archive']).read_bytes();b=gzip.decompress(b) if x.get('encoding')=='gzip' else b;oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert (hashlib.sha256(b).hexdigest(),oid,tree)==(x['sha256'],x['blob'],x['blob']);proof.append(dict(x,commit_tree=root,chain=chain))
(p/'git-proofs.json').write_text(json.dumps({'objects':objs,'proofs':proof,'qualification':'6533 and 95e assigned project objects match per file; f82 is evidence transport only. Pr-working tree had unrelated modifications.'},indent=2)+'\n');print('proofs',len(proof),'objects',len(objs))
