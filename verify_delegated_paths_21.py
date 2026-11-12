#!/usr/bin/env python3
"""Verify public packet Git path proofs without granting source-read credit."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent; cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-21.json').read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree(b):
 out={};i=0
 while i<len(b):
  z=b.index(b'\0',i);mode,name=b[i:z].split(b' ',1);out[name.decode()]=(mode.decode().lstrip('0'),b[z+1:z+21].hex());i=z+21
 assert i==len(b);return out
changed=[];count=0
for bundle in cp['bundles']:
 members={m['path']:m for m in bundle['members']}
 with tarfile.open(R/bundle['archive']) as t:
  d=json.load(t.extractfile('git-proofs.json'));objects={}
  for k,x in d['objects'].items():
   p=x.get('archive',x.get('preserved_bytes'));m=members[p];assert not m['changed'];b=gzip.decompress(t.extractfile(p).read());assert len(b)==x['bytes'] and sha(b)==x['sha256'] and oid(x['type'],b)==k;objects[k]=b
  for x in d.get('proofs',d.get('paths',[])):
   rev=x['revision'];commit=objects[rev];root=commit.splitlines()[0].split()[1].decode();assert root==x.get('commit_tree',d.get('commits',{}).get(rev,{}).get('root_tree'))
   chain=x.get('chain',x.get('path_chain'));parts=x['path'].split('/');assert len(parts)==len(chain);cur=root
   for n,s in zip(parts,chain):
    assert s['tree']==cur and s['name']==n;mode,child=tree(objects[cur])[n];assert mode==s['mode'].lstrip('0') and child==s['object'];cur=child
   assert cur==x.get('blob',x.get('git_blob',cur))
   p=x.get('archive',x.get('local',x.get('preserved_bytes')));m=members[p];raw=t.extractfile(p).read();raw=gzip.decompress(raw) if m['content_encoding']=='gzip' else raw
   if m['changed']:
    assert m['original_uncompressed_sha256']==x.get('sha256') and 'Reconstructed public copy' in m['qualification'];changed.append((bundle['bundle_id'],p))
   else:
    assert oid('blob',raw)==cur
    if x.get('sha256'):assert sha(raw)==x['sha256']
    if x.get('bytes') is not None:assert len(raw)==x['bytes']
   count+=1
assert changed==[('shell-fixture-2664','history-774/contract-groups.jsonl.gz')]
print('Verified',count,'raw Git path chains; one redacted 774 body is qualified by its original digest, with exact top-level original retained only locally. No source-read credit inferred.')
