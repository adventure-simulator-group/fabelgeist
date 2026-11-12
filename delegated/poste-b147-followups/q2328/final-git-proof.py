import json,pathlib,hashlib,datetime
s=pathlib.Path(__file__).parent; reg=json.loads((s/'TREE-DISCOVERY-REGISTRATION.json').read_text()); out=[]; used=set()
(s/'FINAL-GIT-PROOF-REGISTRATION.json').write_text(json.dumps({'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All eleven listed revision/path pairs from TREE-DISCOVERY-REGISTRATION; only source paths, Git object identities and raw-commit digests, no commit metadata','roots':'raw commit external anchors','tree_verification':'Git SHA-1 type/length/object bytes','blob_verification':'Git SHA-1 type/length/source bytes'},indent=2)+'\n')
for rev,key,anchor in [('95e63628f9d70b8dd6e6fede64334696c6995009','95e_paths',reg['anchors'][0]),('e2755a732c84e67296b5e3441a4259a84f461b2f','e275_paths',reg['anchors'][1])]:
 b=pathlib.Path(anchor).read_bytes();assert hashlib.sha1(b'commit '+str(len(b)).encode()+b'\0'+b).hexdigest()==rev;root=b.split(b'\n',1)[0].split()[1].decode()
 for path in reg[key]:
  cur=root; chain=[];parts=path.split('/')
  for idx,name in enumerate(parts):
   data=(s/'git-trees'/(cur+'.tree')).read_bytes();assert hashlib.sha1(b'tree '+str(len(data)).encode()+b'\0'+data).hexdigest()==cur;used.add(cur);ents={};i=0
   while i<len(data):
    j=data.index(b' ',i);k=data.index(b'\0',j+1);ents[data[j+1:k].decode()]=(data[i:j].decode(),data[k+1:k+21].hex());i=k+21
   mode,nxt=ents[name];chain.append({'tree':cur,'component':name,'mode':mode,'object':nxt});cur=nxt
   if idx<len(parts)-1: assert mode=='40000'
   else:
    assert mode in ('100644','100755'); src=(s/'sources'/rev/path).read_bytes(); assert hashlib.sha1(b'blob '+str(len(src)).encode()+b'\0'+src).hexdigest()==cur
    out.append({'revision':rev,'commit_sha256':hashlib.sha256(b).hexdigest(),'commit_bytes':len(b),'root_tree':root,'path':path,'source_sha256':hashlib.sha256(src).hexdigest(),'source_bytes':len(src),'chain':chain})
(s/'FINAL-GIT-PROOF.json').write_text(json.dumps(out,indent=2)+'\n');(s/'TREE-USED.json').write_text(json.dumps(sorted(used),indent=2)+'\n');print(json.dumps({'revision_path_pairs':len(out),'tree_objects_used':len(used),'sources_verified':True},indent=2))
