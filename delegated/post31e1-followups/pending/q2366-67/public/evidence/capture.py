import datetime,hashlib,json,pathlib,subprocess,traceback
base=pathlib.Path(__file__).resolve().parent;reg=json.loads((base/'REGISTRATION-006.json').read_text());repo=reg['git_source']['repo']
def sh(b):return hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def write(path,b):
 path.parent.mkdir(parents=True,exist_ok=True)
 with path.open('xb') as f:f.write(b)
def jwrite(path,obj):write(path,(json.dumps(obj,indent=2,sort_keys=True)+'\n').encode())
def git(*args):
 p=subprocess.run(['git','-C',repo,*args],capture_output=True)
 if p.returncode:raise RuntimeError('git '+str(args)+' exit '+str(p.returncode)+' stderr '+p.stderr.decode(errors='replace'))
 return p.stdout
def tree_entries(b):
 out={};i=0
 while i<len(b):
  a=b.index(b' ',i);z=b.index(b'\0',a);out[b[a+1:z].decode()]=(b[i:a].decode(),b[z+1:z+21].hex());i=z+21
 return out
try:
 entries=[json.loads((base/f'q{n}-corrected-entry-0.json').read_bytes()) for n in (2366,2367)];refs={};expected={}
 def add(path,sha):
  path=path.removeprefix('<PRIVATE-PATH-0022>')
  if path in refs:assert refs[path]==sha
  refs[path]=sha
 for e in entries:
  for row in e['exact_source_identities_and_assessed_ranges']:
   path=row['source_id'].split(':')[2]
   if path in expected:assert expected[path]==row['source_sha256']
   expected[path]=row['source_sha256']
   for k in ('body_receipt','registration_ref_and_sha256'):add(row[k]['path'],row[k]['sha256'])
   add(row['root_adoption']['path'],row['root_adoption']['sha256']);add(row['root_adoption']['verification'],row['root_adoption']['verification_sha256'])
  add(e['proposal']['path'],e['proposal']['sha256']);c=e['related_prior_native_policy_context']['bundle'];add(c['path'],c['sha256'])
 history=[];root=pathlib.Path(reg['history']['root'])
 for path,sha in sorted(refs.items()):
  b=(root/path).read_bytes();assert sh(b)==sha,(path,sh(b),sha);write(base/'data/history'/path,b);history.append({'path':path,'bytes':len(b),'sha256':sh(b),'matched_recorded_sha256':True})
 jwrite(base/'HISTORY-COPY-RECEIPT.json',history)
 proofs=[];tree_saved=set();by_path={}
 for rev in reg['git_source']['revisions']:
  commit=git('cat-file','commit',rev);assert oid('commit',commit)==rev
  root_oid=commit.split(b'\n',1)[0].split()[1].decode();label='95e' if rev.startswith('95e') else 'e275-reconstructed'
  for path in reg['git_source']['paths']:
   cur=root_oid;chain=[]
   for component in path.split('/'):
    tree=git('cat-file','tree',cur);assert oid('tree',tree)==cur
    if cur not in tree_saved:write(base/'data/proof/trees'/(cur+'.tree'),tree);tree_saved.add(cur)
    mode,nxt=tree_entries(tree)[component];chain.append({'tree':cur,'component':component,'mode':mode,'oid':nxt});cur=nxt
   b=git('cat-file','blob',cur);assert oid('blob',b)==cur;assert sh(b)==expected[path],(rev,path,sh(b),expected[path]);write(base/'data/sources'/label/path,b)
   x={'revision':rev,'path':path,'chain':chain,'blob':cur,'source_sha256':sh(b),'bytes':len(b),'recorded_e275_sha256':expected[path],'historical_record_hash_match':True};proofs.append(x);by_path.setdefault(path,{})[label]=sh(b)
 assert all(v['95e']==v['e275-reconstructed'] for v in by_path.values())
 jwrite(base/'SOURCE-PROOF-RECEIPT.json',{'paths':proofs,'per_file_current_and_historical_hash_equal':True,'whole_tree_equality_claimed':False,'historical_body_qualification':'e275 bytes reconstructed through proven Git blob, not recovered historical reviewer read body'})
 print(json.dumps({'history_records':len(history),'proof_paths':len(proofs),'raw_tree_objects':len(tree_saved),'all_historical_hashes_match':True,'per_file_equal':True},indent=2))
except Exception as exc:
 jwrite(base/'CAPTURE-ERROR.json',{'error':repr(exc),'traceback':traceback.format_exc(),'qualification':'Partial outputs retained. No retries overwrite previous files.'});raise
