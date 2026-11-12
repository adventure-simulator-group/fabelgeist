#!/usr/bin/env python3
"""Capture source tree membership without raw commit author metadata."""
import argparse,hashlib,json,pathlib,subprocess
p=pathlib.Path(__file__).resolve().parent;arg=argparse.ArgumentParser();arg.add_argument('--repo',required=True);repo=arg.parse_args().repo
def git(*a):return subprocess.check_output(['git','-C',repo,*a])
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def find(t,n):
 for x in git('ls-tree','-z',t).split(b'\0'):
  if not x:continue
  h,name=x.split(b'\t',1)
  if name.decode()==n:return h.decode().split()
 raise ValueError((t,n))
items=[]
for f in ['SOURCE-REGISTRATION.json','SUPPORT-REGISTRATION.json','SUPPORT-UI-REGISTRATION.json']:
 items.extend(json.loads((p/f).read_text())['items'])
items.append(json.loads((p/'NATIVE-CAPTURE.json').read_text())['items'][0])
g=json.loads((p/'L13-PRIMARY-CAPTURE.json').read_text())['git_comparison'];items.append({**g,'file':g['capture']})
proofs=[]
for x in items:
 if not x.get('present',True):continue
 rev=x['revision'];path=x['path'];raw=(p/x['file']).read_bytes();root=git('rev-parse',rev+'^{tree}').decode().strip();current=root;chain=[];parts=path.split('/')
 for i,part in enumerate(parts):
  b=git('cat-file','tree',current)
  if oid('tree',b)!=current:raise ValueError('tree object mismatch')
  dest=p/'git-trees'/current;dest.parent.mkdir(exist_ok=True);dest.write_bytes(b)
  mode,kind,child=find(current,part);chain.append({'tree':current,'name':part,'mode':mode,'kind':kind,'object':child})
  if i<len(parts)-1:
   if kind!='tree':raise ValueError('not tree')
   current=child
  elif kind!='blob' or oid('blob',raw)!=child:raise ValueError('blob mismatch')
 proofs.append({'revision':rev,'revision_tree':root,'path':path,'file':x['file'],'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'chain':chain})
(p/'GIT-TREE-PROOFS.json').write_text(json.dumps({'schema':'726-q2330-tree-proofs-v1','proofs':proofs,'qualification':'Raw commit objects and author metadata omitted; independent commit-to-tree binding requires the named Git objects. Tree proof does not imply body read credit.'},indent=2)+'\n');print('proofs',len(proofs),'trees',len(list((p/'git-trees').iterdir())))
