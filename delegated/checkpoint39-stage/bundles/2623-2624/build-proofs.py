from pathlib import Path
import gzip,hashlib,json,subprocess
b=Path(__file__).resolve().parent;repo='<PRIVATE_PATH_0001>';sha=lambda d:hashlib.sha256(d).hexdigest();oid=lambda t,d:hashlib.sha1(t.encode()+b' '+str(len(d)).encode()+b'\0'+d).hexdigest();g=lambda *a:subprocess.check_output(['git','-C',repo,*a]);objects={};commits={};proofs=[]
def save(t,k):
 if k not in objects:
  d=g('cat-file',t,k);assert oid(t,d)==k;p='git-objects/'+k+'.'+t+'.gz';(b/p).parent.mkdir(exist_ok=True);(b/p).write_bytes(gzip.compress(d,mtime=0));objects[k]={'type':t,'sha256':sha(d),'bytes':len(d),'preserved_bytes':p}
 return gzip.decompress((b/objects[k]['preserved_bytes']).read_bytes())
def ents(raw):
 out={};i=0
 while i<len(raw):
  end=raw.index(b'\0',i);mode,name=raw[i:end].split(b' ',1);out[name.decode()]=(mode.decode(),raw[end+1:end+21].hex());i=end+21
 return out
def root(rev):
 if rev not in commits:
  d=save('commit',rev);commits[rev]={'root_tree':d.splitlines()[0].split()[1].decode(),'parents':[x.split()[1].decode() for x in d.splitlines() if x.startswith(b'parent ')]}
 return commits[rev]['root_tree']
def proof(rev,path,file,kind,compressed=False):
 d=(b/file).read_bytes();d=gzip.decompress(d) if compressed else d;cur=root(rev);chain=[]
 for seg in path.split('/'):
  mode,child=ents(save('tree',cur))[seg];chain.append({'tree':cur,'name':seg,'mode':mode,'object':child});cur=child
 assert oid('blob',d)==cur;proofs.append({'revision':rev,'path':path,'kind':kind,'sha256':sha(d),'bytes':len(d),'git_blob':cur,'preserved_bytes':file,'preserved_encoding':'gzip' if compressed else 'raw','path_chain':chain})
f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
for f in ['registration.json','supplement-registration.json','root-registration.json','warmup-registration.json']:
 for x in json.loads((b/f).read_bytes())['entries']:
  proof(x['revision'],x['path'],x['archive'],'project per-file Git identity; actual source windows and historical dirty tree separately qualified')
  if x['id'] in ('garment-old','shape-old'):proof(f82,'handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/'+x['sha256'],x['archive'],'original historical source object transported by f82, not source revision')
for x in json.loads((b/'history/records.json').read_bytes())['records']:
 if x['kind']=='774-f82':proof(f82,x['transport_path'],x['archive'],'original 774 evidence transported by f82',True)
rev=json.loads((b/'history/publication-capture.json').read_bytes())['commit']
for f in ['BASELINE-95E-20261110.json','ASSESSMENT-STAGING-TASSETS-95E.json','ASSESSMENT-L01-TASSET-NODE-95E.json','DELEGATED-EVIDENCE-CHECKPOINT-17.json','DELEGATED-CHECKPOINT-17-QUALIFICATION.json']:proof(rev,f,'history/published/'+f,'published baseline, current assessment or checkpoint metadata')
for p,f in [('recovered/original-transport-manifest.json.gz','history/original-transport-manifest.json.gz'),('recovered/original-registry-source-archive/registry.json.gz','history/original-registry.json.gz'),('recovered/physical-p16-patch-recovery/tracked.patch','history/p16-tracked.patch'),('delegated/armor-garment-shape-evidence.tar.gz','history/prior-role/published-evidence.tar.gz')]:proof(rev,p,f,'published source transport or archive with recovery qualifications')
(b/'git-proofs.json').write_text(json.dumps({'schema':'726-garment-shape-reconciliation-git-proof-v1','commits':commits,'objects':objects,'paths':proofs,'qualification':'Per-file and transport proofs do not establish historical whole-tree cleanliness or transfer prior readers.'},indent=2)+'\n');print('git proofs',len(proofs),'objects',len(objects))
