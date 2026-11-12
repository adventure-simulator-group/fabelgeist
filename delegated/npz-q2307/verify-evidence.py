#!/usr/bin/env python3
"""Verify the preserved NPZ evidence; never invoke product code."""
import argparse,gzip,hashlib,json
from datetime import datetime
from pathlib import Path
p=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();group=ap.add_mutually_exclusive_group(required=True);group.add_argument('--anchors-dir',type=Path);group.add_argument('--publication-root',type=Path);ap.add_argument('--original-queue',type=Path);ap.add_argument('--check-metadata',action='store_true');a=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def gid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(n):return json.loads((p/n).read_bytes())
reg=load('SOURCE-REGISTRATION.json');hreg=load('HISTORY-REGISTRATION.json');mreg=load('METADATA-HISTORY-REGISTRATION.json');rreg=load('RELATED-HISTORY-REGISTRATION.json');sread=load('SOURCE-READ-RECEIPT.json');hread=load('HISTORY-READ-RECEIPT.json');anchor=load('ANCHORS.json');extract=load('QUEUE-EXTRACTION.json');related=load('RELATED-EXTRACTION.json');q=load('QUEUE-ENTRY-ORIGINAL.json')
assert sread['registration_sha256']==sha((p/'SOURCE-REGISTRATION.json').read_bytes())
assert all(hread['registrations'][n]==sha((p/n).read_bytes()) for n in hread['registrations'])
roots={}
for oid,meta in anchor['commits'].items():
 if a.anchors_dir:data=(a.anchors_dir/(oid+'.commit')).read_bytes()
 else:
  data=(a.publication_root/anchor['locations_in_existing_publication'][oid]).read_bytes()
  if data.startswith(b'\x1f\x8b'):data=gzip.decompress(data)
 assert gid('commit',data)==oid and sha(data)==meta['raw_sha256'] and len(data)==meta['bytes']
 roots[oid]=data.splitlines()[0].decode().split(' ')[1];assert roots[oid]==meta['root_tree']==reg['commit_roots'][oid]
def entries(oid):
 data=(p/'git-trees'/(oid+'.tree')).read_bytes();assert gid('tree',data)==oid
 at=0;out={}
 while at<len(data):
  z=data.index(b'\0',at);mode,name=data[at:z].split(b' ',1);child=data[z+1:z+21].hex();at=z+21;assert len(child)==40 and name.decode() not in out;out[name.decode()]=(mode.decode(),child)
 assert at==len(data);return out
def lookup(root,path):
 node=root;parts=path.split('/')
 for i,part in enumerate(parts):
  mode,node=entries(node)[part]
  if i<len(parts)-1:assert mode=='40000'
 return mode,node
cur=reg['baseline'];old=reg['historical_revision']
assert lookup(roots[cur],'crates/fabelgeist-numpy-storage/src')==lookup(roots[old],'crates/fabelgeist-numpy-storage/src')==('40000',reg['source_subtree_both'])
assert lookup(roots[cur],'crates/fabelgeist-numpy-storage')[1]==reg['crate_tree_current'] and lookup(roots[old],'crates/fabelgeist-numpy-storage')[1]==reg['crate_tree_historical'] and roots[cur]!=roots[old]
x=entries(reg['crate_tree_current']);y=entries(reg['crate_tree_historical']);assert {k for k in x.keys()|y.keys() if x.get(k)!=y.get(k)}=={'Cargo.toml'}
rows={r['path']:r for r in reg['rows']};actual={name:set() for name in rows}
for name,row in rows.items():
 data=(p/row['source_file']).read_bytes();assert sha(data)==row['sha256'] and gid('blob',data)==row['git_blob'] and len(data)==row['bytes'] and len(data.splitlines())==row['line_count']
 assert lookup(roots[cur],name)==lookup(roots[old],name)==('100644',row['git_blob'])
for d in sread['displays']:
 out=[];assert datetime.fromisoformat(d['observed_mtime_utc'])>=datetime.fromisoformat(reg['registered_utc'])
 for view in d['rows']:
  path=view['path'];lo,hi=view['range'];row=rows[path];lines=(p/row['source_file']).read_text().splitlines();assert any(x<=lo<=hi<=y for x,y in row['registered_ranges'])
  out.append(f"=== {path} {lo}..{hi} ===");out.extend(f"{i:4d} | {lines[i-1]}" for i in range(lo,hi+1));new=set(range(lo,hi+1));assert not actual[path]&new;actual[path]|=new
 data=('\n'.join(out)+'\n').encode();assert data==(p/d['file']).read_bytes() and sha(data)==d['sha256'] and len(data)==d['bytes']
for path,row in rows.items():
 wanted=set()
 for x,y in row['registered_ranges']:wanted.update(range(x,y+1))
 assert wanted==actual[path];unread=set(range(1,row['line_count']+1))-wanted;recorded=set()
 for x,y in sread['unread_complements'].get(path,[]):recorded.update(range(x,y+1))
 assert unread==recorded
for record in mreg['historical_records']:
 data=(p/record['copy']).read_bytes();assert len(data)==record['bytes'] and sha(data)==record['sha256']
for v in hread['views']:
 data=(p/v['file']).read_bytes();assert len(data)==v['bytes'] and sha(data)==v['sha256'];name=v['file'];registration='HISTORY-REGISTRATION.json' if name.startswith('HISTORY-DISPLAY-') else ('RELATED-HISTORY-REGISTRATION.json' if name=='RELATED-HISTORY-DISPLAY.json' else 'METADATA-HISTORY-REGISTRATION.json')
 assert datetime.fromisoformat(v['observed_mtime_utc'])>=datetime.fromisoformat(load(registration)['registered_utc'])
original=(p/'QUEUE-ENTRY-ORIGINAL.json').read_bytes();rec=(p/'QUEUE-ENTRY-RECONSTRUCTION.json').read_bytes();assert original==rec and extract['entry_bytes_equal'] and sha(original)==extract['records']['original']['entry_sha256']==extract['records']['reconstruction']['entry_sha256']
assert q['contract_id']=='REVIEW-L11-CURRENT-NPZ-SELECTED-MEMBER-AND-DECLARATION-ROLES' and not q['implementation_gate_complete'] and not q['audit_discovery_closed_within_exact_named_scope'] and q['body_or_population_credit']==0
for row in q['exact_source_identities_and_assessed_ranges']:
 path=row['source_id'].split(':')[2];assert path in rows and row['source_sha256']==rows[path]['sha256'] and row['revision_or_explicit_transport_qualification']==old
for f,lo,hi in [('HISTORY-DISPLAY-A.txt',1,85),('HISTORY-DISPLAY-B.txt',86,170),('HISTORY-DISPLAY-C.txt',171,255),('HISTORY-DISPLAY-D.txt',256,340)]:
 lines=original.decode().splitlines();out=f'=== QUEUE-ENTRY-ORIGINAL.json {lo}..{hi} ===\n'+'\n'.join(f'{i:4d} | {lines[i-1]}' for i in range(lo,hi+1))+'\n';assert (p/f).read_text()==out
hist=p/'historical-originals';source='parallel-l11-20261029/READ-171.json';d=json.loads((hist/source).read_bytes())
for n in ['A','B']:
 z=load('HISTORY-PROJECTION-'+n+'.json');assert z['source']==source
 for ptr,val in z['views'].items():
  curval=d
  for part in ptr.strip('/').split('/'):curval=curval[int(part)] if isinstance(curval,list) else curval[part]
  assert curval==val
z=load('HISTORY-PROJECTION-C.json');ad=json.loads((hist/'segovb/READ-L11-707.json').read_bytes());pr=json.loads((hist/'parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-003.json').read_bytes())['contract_updates'][7];assert all(ad[k]==v for k,v in z['root_adoption'].items()) and all(pr[k]==v for k,v in z['proposal_contract_7'].items())
rdisp=load('RELATED-HISTORY-DISPLAY.json')['entries']
for r in rdisp:
 k=str(r['index']);data=(p/'related-original'/('q'+k+'.json')).read_bytes();assert sha(data)==related['entries'][k]['entry_sha256'] and len(data)==related['entries'][k]['bytes'];d=json.loads(data);assert all(d[x]==v for x,v in r['fields'].items())
if a.original_queue:
 raw=a.original_queue.read_bytes();e=extract['records']['original'];assert sha(raw)==e['queue_sha256'] and len(raw)==e['queue_bytes'] and raw[e['byte_start']:e['byte_end_exclusive']]==original and sha(raw)==related['original_queue_sha256']
 for k,v in related['entries'].items():assert raw[v['start']:v['end_exclusive']]==(p/'related-original'/('q'+k+'.json')).read_bytes()
 print('original 059 full-queue hash and ten exact entry slices verified')
else:print('original full queue not supplied; entry slices checked only against recorded digests')
if a.check_metadata:
 assert a.publication_root
 search=load('METADATA-SEARCH.json');qraw=(a.publication_root/'baseline-059/queue.json').read_bytes();braw=(a.publication_root/'BASELINE-95E-20261110.json').read_bytes();assert sha(qraw)==search['historical_reconstruction']['source_sha256'] and sha(braw)==search['current_working_ledger']['source_sha256']
 qrows=json.loads(qraw)['entries'];out=[]
 for i,r in enumerate(qrows):
  if isinstance(r,dict):
   hay=' '.join(str(r.get(k,'')) for k in search['historical_reconstruction']['fields']).lower()
   if any(t in hay for t in search['historical_reconstruction']['terms']):out.append({'index':i,'contract_id':r.get('contract_id'),'kind':r.get('kind'),'lens':r.get('lens')})
 assert len(out)==search['historical_reconstruction']['total_matches'] and out[:50]==search['historical_reconstruction']['first_50_displayed']
 ledger=json.loads(braw)['newly_assessed_ledger']['entries'];found=[]
 for i,r in enumerate(ledger):
  if isinstance(r,dict) and any(t in json.dumps(r,ensure_ascii=True).lower() for t in search['current_working_ledger']['case_insensitive_terms']):found.append({'index':i,'id':r.get('id') or r.get('contract_id') or r.get('finding_id')})
 assert len(ledger)==search['current_working_ledger']['entries'] and found==search['current_working_ledger']['matches'];print('bounded metadata search replayed against the exact captured identities')
else:print('time-bound metadata search not replayed')
if (p/'MANIFEST.json').exists():
 manifest=load('MANIFEST.json');files={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert files==set(manifest['files']) and all(not x.is_symlink() for x in p.rglob('*'))
 for name,meta in manifest['files'].items():
  data=(p/name).read_bytes();assert len(data)==meta['bytes'] and sha(data)==meta['sha256']
print('commit/tree proofs, eight source blobs, registered displays, original history and selected projections verified')
print('semantic assessment, product behavior, and privacy beyond separate bounded scans not verified')
