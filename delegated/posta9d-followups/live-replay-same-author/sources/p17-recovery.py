import gzip,hashlib,io,json,os,subprocess,tarfile,tempfile
from pathlib import Path
os.environ['GIT_CONFIG_NOSYSTEM']='1';os.environ['GIT_CONFIG_GLOBAL']='/dev/null';os.environ['GIT_ATTR_NOSYSTEM']='1'
R=Path(__file__).resolve().parent;O=R/'recovered/p17-patch-recovery';record=json.loads((R/'RECOVERY-REPORT-P17-PATCH-20261112.json').read_text());raw=(O/'git-delta-proof.tar.gz').read_bytes();sha=lambda b:hashlib.sha256(b).hexdigest();oid=lambda k,b:hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 files=t.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files};assert len(files)==len(data)
assert (sha(raw),len(raw))==(record['git_tree_delta_proof']['sha256'],record['git_tree_delta_proof']['bytes'])
d=json.loads(data['proof.json']);objects={}
for h,x in d['objects'].items():
 b=data[x['archive']];assert len(b)==x['bytes'] and sha(b)==x['sha256'] and oid(x['type'],b)==h;objects[h]=(x['type'],b)
assert set(data)=={'proof.json'}|{x['archive'] for x in d['objects'].values()}
for k in ('base','head'):assert objects[d[k]][1].splitlines()[0]==b'tree '+d[k+'_tree'].encode()
def ents(b):
 out={};i=0
 while i<len(b):
  z=b.index(b'\0',i);mode,name=b[i:z].split(b' ',1);out[name.decode()]=(mode.decode().lstrip('0'),b[z+1:z+21].hex());i=z+21
 assert i==len(b);return out
leaves=[]
def walk(a,b,path):
 left=ents(objects[a][1]) if a else {};right=ents(objects[b][1]) if b else {}
 for name in sorted(left.keys()|right.keys()):
  x=left.get(name);y=right.get(name)
  if x==y:continue
  p=path+name
  if (x and x[0]=='40000') or (y and y[0]=='40000'):
   assert not x or x[0]=='40000';assert not y or y[0]=='40000';walk(x[1] if x else None,y[1] if y else None,p+'/')
  else:
   for q in (x,y):
    if q:assert q[0]=='100644' and objects[q[1]][0]=='blob'
   leaves.append({'path':p,'base':{'mode':x[0],'blob':x[1]} if x else None,'head':{'mode':y[0],'blob':y[1]} if y else None})
walk(d['base_tree'],d['head_tree'],'');assert leaves==d['changed_paths'] and len(leaves)==32
with tempfile.TemporaryDirectory(prefix='726-p17-proof-') as tmp:
 subprocess.run(['git','init','--bare','--quiet',tmp],check=True)
 for h,(k,b) in sorted(objects.items(),key=lambda pair: {'blob':0,'tree':1,'commit':2}[pair[1][0]]):
  p=subprocess.run(['git','--git-dir',tmp,'hash-object','-w','-t',k,'--stdin'],input=b,stdout=subprocess.PIPE,check=True);assert p.stdout.decode().strip()==h
 out=subprocess.check_output(['git','--git-dir',tmp,'-c','core.quotePath=true','diff','--binary','--abbrev=9',d['base'],d['head']])
registry=json.loads(gzip.decompress((R/record['registry']).read_bytes()));snapshot=next(x for x in registry['snapshots'] if x['id']=='report-p17');assert snapshot['base']==d['base']==record['recorded_base'] and snapshot['head']==d['head']==record['recorded_head'] and snapshot['status_porcelain']==record['recorded_status_porcelain']==[] and snapshot['changed_variants']==record['recorded_changed_variants']==len(leaves) and snapshot['meaning']==record['recorded_meaning']
patch=(O/'tracked.patch').read_bytes();assert snapshot['patch_sha256']==record['recovered_patch']['sha256'] and len(patch)==record['recovered_patch']['bytes'];assert out==patch and len(patch)==268281 and sha(patch)=='3c72c8c4f9816b49ae6306994d55571ceeff420b5a47bb5ff8bcec3c6668c9cd'
print('Verified 85 Git objects, complete 32-path tree delta and exact P17 patch reproduction in a temporary object-only repository; original invocation and current cleanliness remain unknown.')
