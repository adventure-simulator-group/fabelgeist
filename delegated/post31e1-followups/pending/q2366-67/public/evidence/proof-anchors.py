import datetime,hashlib,json,pathlib,subprocess,traceback
base=pathlib.Path(__file__).resolve().parent;r=json.loads((base/'REGISTRATION-006.json').read_text());r2=json.loads((base/'REGISTRATION-009.json').read_text());repo=pathlib.Path(r['git_source']['repo'])
def sha(b):return hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def write(p,b):
 p.parent.mkdir(parents=True,exist_ok=True)
 with p.open('xb') as f:f.write(b)
def jwrite(p,x):write(p,(json.dumps(x,indent=2,sort_keys=True)+'\n').encode())
def parse(b):
 out={};i=0
 while i<len(b):
  a=b.index(b' ',i);z=b.index(b'\0',a);out[b[a+1:z].decode()]=(b[i:a].decode(),b[z+1:z+21].hex());i=z+21
 return out
try:
 entries=[json.loads((base/f'q{n}-corrected-entry-0.json').read_bytes()) for n in (2366,2367)];expected={}
 for entry in entries:
  for row in entry['exact_source_identities_and_assessed_ranges']:
   p=row['source_id'].split(':')[2]
   if p in expected:assert expected[p]==row['source_sha256']
   expected[p]=row['source_sha256']
 roots=r2['actions'][1]['tree_roots'];anchor=pathlib.Path(r2['actions'][0]['directory']);trees={};routes={};fallback_failures=[];proof=[];missing=[]
 def gettree(cur):
  if cur in trees:return trees[cur]
  for prefix in roots:
   p=pathlib.Path(prefix)/(cur+'.tree')
   if p.is_file():b=p.read_bytes();assert oid('tree',b)==cur;routes[cur]='existing-evidence-tree';break
  else:
   p=subprocess.run(['git','-C',str(repo),'cat-file','tree',cur],capture_output=True)
   if p.returncode:
    fallback_failures.append({'tree':cur,'exit':p.returncode,'stderr':p.stderr.decode(errors='replace')});raise FileNotFoundError('missing tree '+cur)
   b=p.stdout;assert oid('tree',b)==cur;routes[cur]='git-cat-file-tree'
  trees[cur]=b;write(base/'data2/proof/trees'/(cur+'.tree'),b);return b
 inventory=pathlib.Path(r2['actions'][3]['inventory']).read_bytes();inv_lines=inventory.decode().splitlines();inv=[]
 for path in r['git_source']['paths']:
  hits=[l for l in inv_lines if l.endswith('\t'+path)];assert len(hits)==1;inv.append({'path':path,'row':hits[0]})
  b=(repo/path).read_bytes();write(base/'data2/source-candidates'/path,b)
  assert sha(b)==expected[path],(path,sha(b),expected[path]);assert oid('blob',b)==hits[0].split()[2]
 for rev in r['git_source']['revisions']:
  commit=(anchor/(rev+'.commit')).read_bytes();assert oid('commit',commit)==rev;root=commit.split(b'\n',1)[0].split()[1].decode()
  for path in r['git_source']['paths']:
   cur=root;chain=[]
   try:
    for component in path.split('/'):
     mode,nxt=parse(gettree(cur))[component];chain.append({'tree':cur,'component':component,'mode':mode,'oid':nxt});cur=nxt
    b=(base/'data2/source-candidates'/path).read_bytes();assert oid('blob',b)==cur;assert sha(b)==expected[path]
    proof.append({'revision':rev,'path':path,'chain':chain,'blob':cur,'source_sha256':sha(b),'bytes':len(b),'recorded_e275_sha256':expected[path],'historical_record_hash_match':True})
   except Exception as ex:missing.append({'revision':rev,'path':path,'chain':chain,'error':repr(ex)})
 result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inventory_sha256':sha(inventory),'inventory_rows':inv,'proofs':proof,'missing':missing,'tree_routes':routes,'fallback_failures':fallback_failures,'qualification':'Physical current candidates are authoritative only where the 95e commit-to-tree-to-blob and inventory match. Equal e275 paths prove per-file identity; e275 source bytes are reconstructed, not recovered historical reviewer body. No whole-tree equality.'}
 jwrite(base/'PROOF-ANCHORS-RESULT.json',result)
 print(json.dumps({'proof_count':len(proof),'missing':missing,'fallback_failures':fallback_failures,'trees':len(trees),'source_candidates':len(inv)},indent=2))
except Exception as exc:
 jwrite(base/'PROOF-ANCHORS-ERROR.json',{'error':repr(exc),'traceback':traceback.format_exc(),'qualification':'Partial outputs retained; no retry may overwrite existing paths.'});raise
