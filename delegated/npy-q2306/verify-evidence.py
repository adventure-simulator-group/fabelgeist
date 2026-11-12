#!/usr/bin/env python3
"""Verify preserved source/history evidence only; never invoke product code."""
import argparse,gzip,hashlib,json,re
from datetime import datetime
from pathlib import Path
p=Path(__file__).resolve().parent
ap=argparse.ArgumentParser()
g=ap.add_mutually_exclusive_group(required=True)
g.add_argument("--anchors-dir",type=Path)
g.add_argument("--publication-root",type=Path)
ap.add_argument("--original-queue",type=Path,help="optional original 059 queue; checked without copying")
a=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def gid(kind,b):return hashlib.sha1(kind.encode()+b" "+str(len(b)).encode()+b"\0"+b).hexdigest()
def load(n):return json.loads((p/n).read_bytes())
reg=load("SOURCE-REGISTRATION.json");hreg=load("HISTORY-REGISTRATION.json");hext=load("HISTORY-EXT-REGISTRATION.json");hread=load("HISTORY-READ-RECEIPT.json");sread=load("SOURCE-READ-RECEIPT.json");anchor=load("ANCHORS.json");extract=load("QUEUE-EXTRACTION.json");q=load("HISTORY-Q2306-ORIGINAL-EXACT.json")
assert hext["prior_registration_sha256"]==sha((p/"HISTORY-REGISTRATION.json").read_bytes())
assert sread["registration_sha256"]==sha((p/"SOURCE-REGISTRATION.json").read_bytes())
assert all(hread["registrations"][n]==sha((p/n).read_bytes()) for n in hread["registrations"])
roots={}
for oid,meta in anchor["commits"].items():
 if a.anchors_dir: data=(a.anchors_dir/(oid+".commit")).read_bytes()
 else:
  data=(a.publication_root/anchor["locations_in_existing_publication"][oid]).read_bytes()
  if data.startswith(b"\x1f\x8b"):data=gzip.decompress(data)
 assert gid("commit",data)==oid and sha(data)==meta["raw_sha256"] and len(data)==meta["bytes"]
 roots[oid]=data.splitlines()[0].decode().split(" ")[1];assert roots[oid]==meta["root_tree"]==reg["commit_roots"][oid]
def entries(oid):
 data=(p/"git-trees"/(oid+".tree")).read_bytes();assert gid("tree",data)==oid
 at=0;out={}
 while at<len(data):
  z=data.index(b"\0",at);mode,name=data[at:z].split(b" ",1);child=data[z+1:z+21].hex();at=z+21
  assert len(child)==40 and name.decode() not in out;out[name.decode()]=(mode.decode(),child)
 assert at==len(data);return out
def lookup(root,path):
 node=root
 for idx,part in enumerate(path.split("/")):
  mode,node=entries(node)[part]
  if idx<len(path.split("/"))-1:assert mode=="40000"
 return mode,node
cur=reg["baseline"];old=reg["historical_revision"]
assert lookup(roots[cur],"crates/fabelgeist-numpy-storage/src")==lookup(roots[old],"crates/fabelgeist-numpy-storage/src")==('40000',reg['source_subtree_both'])
assert lookup(roots[cur],"crates/fabelgeist-numpy-storage")[1]==reg['crate_tree_current']
assert lookup(roots[old],"crates/fabelgeist-numpy-storage")[1]==reg['crate_tree_historical']
assert reg['crate_tree_current']!=reg['crate_tree_historical'] and roots[cur]!=roots[old]
old_entries=entries(reg['crate_tree_historical']);new_entries=entries(reg['crate_tree_current'])
assert {x for x in old_entries.keys()|new_entries.keys() if old_entries.get(x)!=new_entries.get(x)}=={'Cargo.toml'}
rows={r['path']:r for r in reg['rows']};actual={name:set() for name in rows}
for name,row in rows.items():
 data=(p/row['source_file']).read_bytes();assert sha(data)==row['sha256'] and gid('blob',data)==row['git_blob'] and len(data)==row['bytes'] and len(data.splitlines())==row['line_count']
 assert lookup(roots[cur],name)==lookup(roots[old],name)==('100644',row['git_blob'])
for d in sread['displays']:
 out=[]
 assert datetime.fromisoformat(d['observed_mtime_utc'])>=datetime.fromisoformat(reg['registered_utc'])
 for view in d['rows']:
  path=view['path'];lo,hi=view['range'];row=rows[path];lines=(p/row['source_file']).read_text().splitlines()
  assert any(x<=lo<=hi<=y for x,y in row['registered_ranges'])
  out.append(f"=== {path} {lo}..{hi} ===");out.extend(f"{i:4d} | {lines[i-1]}" for i in range(lo,hi+1))
  new=set(range(lo,hi+1));assert not actual[path]&new;actual[path]|=new
 body=("\n".join(out)+"\n").encode();assert (p/d['file']).read_bytes()==body and sha(body)==d['sha256'] and len(body)==d['bytes']
for path,row in rows.items():
 expect=set()
 for x,y in row['registered_ranges']:expect.update(range(x,y+1))
 assert expect==actual[path]
 unread=set(range(1,row['line_count']+1))-expect;recorded=set()
 for x,y in sread['unread_complements'].get(path,[]):recorded.update(range(x,y+1))
 assert unread==recorded
for record in hext['records']:
 data=(p/record['retained']).read_bytes();assert len(data)==record['bytes'] and sha(data)==record['sha256']
for v in hread['views']:
 data=(p/v['file']).read_bytes();assert len(data)==v['bytes'] and sha(data)==v['sha256']
 source='HISTORY-REGISTRATION.json' if v['file'] in ['HISTORY-DISPLAY-A.txt','HISTORY-DISPLAY-B.txt','HISTORY-DISPLAY-C.txt'] else 'HISTORY-EXT-REGISTRATION.json'
 assert datetime.fromisoformat(v['observed_mtime_utc'])>=datetime.fromisoformat(load(source)['registered_utc'])
original=(p/'HISTORY-Q2306-ORIGINAL-EXACT.json').read_bytes();public=(p/'HISTORY-Q2306-EXACT.json').read_bytes()
assert original==public and sha(original)==extract['original']['entry_sha256']==extract['reconstructed_public']['entry_sha256']
assert q['contract_id']=='REVIEW-L11-CURRENT-OWNED-NPY-HEADER-FRAMING-ADMISSION' and not q['implementation_gate_complete'] and not q['audit_discovery_closed_within_exact_named_scope'] and q['body_or_population_credit']==0
for row in q['exact_source_identities_and_assessed_ranges']:
 path=row['source_id'].split(':')[2];assert path in rows and row['source_sha256']==rows[path]['sha256'] and row['revision_or_explicit_transport_qualification']==old
for name,source,segments in [('HISTORY-DISPLAY-A.txt','HISTORY-Q2306-EXACT.json',[(1,100)]),('HISTORY-DISPLAY-B.txt','HISTORY-Q2306-EXACT.json',[(101,200)]),('HISTORY-DISPLAY-C.txt','HISTORY-Q2306-EXACT.json',[(201,299)]),('HISTORY-ORIGINAL-DISPLAY.txt','HISTORY-Q2306-ORIGINAL-EXACT.json',[(1,21),(262,299)])]:
 lines=(p/source).read_text().splitlines();out=[]
 for lo,hi in segments:out.append(f"=== {source} {lo}..{hi} ===");out.extend(f"{i:4d} | {lines[i-1]}" for i in range(lo,hi+1))
 assert (p/name).read_text()=="\n".join(out)+"\n"
hist=p/'historical-originals'
for name,file in [('A','parallel-l11-20261029/READ-170.json'),('B','parallel-l11-20261029/READ-011.json')]:
 proj=load('HISTORY-PROJECTION-'+name+'.json');d=json.loads((hist/file).read_bytes());assert proj['source']==file
 for pointer,val in proj['views'].items():
  cur=d
  for part in pointer.strip('/').split('/'):cur=cur[int(part)] if isinstance(cur,list) else cur[part]
  assert cur==val
v=load('HISTORY-PROJECTION-C.json')
for z in v:
 d=json.loads((hist/z['source']).read_bytes());assert all(d[k]==value for k,value in z['views'].items())
 for item,full in zip(z['disposition_summaries'],d['finding_connections_and_scope_dispositions']):assert all(item[k]==full.get(k) for k in item)
v=load('HISTORY-PROJECTION-D.json')['views'];d706=json.loads((hist/'segovb/READ-L11-706.json').read_bytes());d547=json.loads((hist/'segovb/READ-L11-547.json').read_bytes());proposal=json.loads((hist/'parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-003.json').read_bytes())['contract_updates'][6]
assert v['706:/finding_connections_and_scope_dispositions/0']==d706['finding_connections_and_scope_dispositions'][0] and v['547:/finding_connections_and_scope_dispositions/0']==d547['finding_connections_and_scope_dispositions'][0]
assert all(proposal[k]==val for k,val in v['proposal:/contract_updates/6'].items())
if a.original_queue:
 data=a.original_queue.read_bytes();assert sha(data)==extract['original']['source_sha256'] and len(data)==extract['original']['source_bytes']
 assert data[extract['original']['byte_start']:extract['original']['byte_end_exclusive']]==original
 print('original 059 full-queue hash and exact byte slice verified')
else:print('original full-queue bytes not supplied; exact slice and recorded hash checked only')
if (p/'MANIFEST.json').exists():
 m=load('MANIFEST.json');files={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert files==set(m['files'])
 assert all(not x.is_symlink() for x in p.rglob('*'))
 for n,meta in m['files'].items():
  data=(p/n).read_bytes();assert len(data)==meta['bytes'] and sha(data)==meta['sha256']
print('e275 and 95e anchors/tree membership, seven source blobs, registrations, displays, historical files and projections verified')
print('semantic assessment, privacy beyond separately stated scan, and product behavior not verified')
