#!/usr/bin/env python3
"""Capture raw Git tree membership, not Git commit metadata or product outputs."""
import argparse,hashlib,json,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
parser=argparse.ArgumentParser();parser.add_argument('--repo',required=True);REPO=parser.parse_args().repo
NOW='a9d979e4f4fd84b797da89a33263d78604fe9224'; OLD='a9b8331ae347542b1ea707b1b04eb8092735d1bd'
def git(*a):return subprocess.check_output(['git','-C',REPO,*a])
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entry(tree,name):
 for v in git('ls-tree','-z',tree).split(b'\0'):
  if not v:continue
  h,n=v.split(b'\t',1)
  if n.decode()==name:
   mode,kind,obj=h.decode().split();return mode,kind,obj
 raise ValueError((tree,name))
def proof(rev,path,content):
 root=git('rev-parse',rev+'^{tree}').decode().strip(); t=root; chain=[]
 for i,part in enumerate(path.split('/')):
  raw=git('cat-file','tree',t);assert oid('tree',raw)==t
  dest=P/'git-trees'/t;dest.parent.mkdir(exist_ok=True);dest.write_bytes(raw)
  mode,kind,child=entry(t,part);chain.append({'tree':t,'name':part,'mode':mode,'kind':kind,'object':child})
  if i<len(path.split('/'))-1:assert kind=='tree';t=child
  else:assert kind=='blob' and oid('blob',content)==child
 return {'revision':rev,'revision_tree':root,'path':path,'sha256':hashlib.sha256(content).hexdigest(),'bytes':len(content),'stored_as':None,'chain':chain}
items=[{'path':'delegated/live-state-successor-design/REPORT-v2.md','stored_as':'proposal.md'}]
for n in (3,4,6,8):
 reg=json.loads((P/f'registration-{n:02d}.json').read_text()); items+=reg['items']
results=[]
for item in items:
 content=(P/item['stored_as']).read_bytes(); record=proof(NOW,item['path'],content);record['stored_as']=item['stored_as'];results.append(record)
 # An old-path equality claim applies only where registration explicitly established it.
 old=item.get('a9b_same_path',{}).get('identical_to_a9d') or item.get('a9b_identical',False)
 if old:
  record=proof(OLD,item['path'],content);record['stored_as']=item['stored_as'];results.append(record)
result={'schema':'live-state-review-git-tree-proofs-v1','proofs':results,'limitations':['Git commit bytes, including author metadata, are not included; independently bind named commit to root tree with the relevant original Git object.','Tree chains and exact source hashes establish file identity, not personal read coverage or whole-tree possession.']}
(P/'TREE-PROOFS.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'proofs':len(results),'unique_tree_objects':len(list((P/'git-trees').iterdir()))},indent=2))
