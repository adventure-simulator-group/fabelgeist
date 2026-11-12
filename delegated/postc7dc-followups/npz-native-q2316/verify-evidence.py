#!/usr/bin/env python3
"""Verify preserved audit evidence bytes only; never execute product code."""
import argparse,gzip,hashlib,json,tarfile,tomllib
from datetime import datetime
from pathlib import Path
p=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();group=ap.add_mutually_exclusive_group(required=True);group.add_argument('--anchors-dir',type=Path);group.add_argument('--publication-root',type=Path);ap.add_argument('--archives-dir',type=Path,required=True);ap.add_argument('--original-queue',type=Path);ap.add_argument('--root-lock',type=Path);ap.add_argument('--ledger',type=Path);a=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def gid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(n):return json.loads((p/n).read_bytes())
regs=[load(n) for n in ['SOURCE-REGISTRATION.json','SOURCE-EXT-REGISTRATION.json','SOURCE-EXT2-REGISTRATION.json']];r0,r1,r2=regs;sr=load('SOURCE-READ-RECEIPT.json');hr=load('HISTORY-READ-RECEIPT.json');anchor=load('ANCHORS.json');hreg=load('NATIVE-HISTORY-REGISTRATION.json');extract=load('QUEUE-EXTRACTION.json');meta=load('DEPENDENCY-METADATA.json');q=load('QUEUE-ENTRY-ORIGINAL.json');safe=load('PUBLIC-SUCCESSOR-QUALIFICATION.json')
assert r1['prior_registration_sha256']==sha((p/'SOURCE-REGISTRATION.json').read_bytes()) and r2['prior_extension_sha256']==sha((p/'SOURCE-EXT-REGISTRATION.json').read_bytes())
assert all(sr['registrations'][n]==sha((p/n).read_bytes()) for n in sr['registrations']) and all(hr['registrations'][n]==sha((p/n).read_bytes()) for n in hr['registrations'])
roots={}
for oid,m in anchor['commits'].items():
 if a.anchors_dir:data=(a.anchors_dir/(oid+'.commit')).read_bytes()
 else:
  data=(a.publication_root/anchor['locations_in_existing_publication'][oid]).read_bytes()
  if data.startswith(b'\x1f\x8b'):data=gzip.decompress(data)
 assert gid('commit',data)==oid and sha(data)==m['raw_sha256'] and len(data)==m['bytes'];roots[oid]=data.splitlines()[0].decode().split(' ')[1];assert roots[oid]==m['root_tree']
assert roots[r0['commit']]==r0['root_tree']
old='e2755a732c84e67296b5e3441a4259a84f461b2f'
def tree(oid):
 data=(p/'git-trees'/(oid+'.tree')).read_bytes();assert gid('tree',data)==oid;out={};i=0
 while i<len(data):
  z=data.index(b'\0',i);mode,name=data[i:z].split(b' ',1);child=data[z+1:z+21].hex();i=z+21;assert len(child)==40 and name.decode() not in out;out[name.decode()]=(mode.decode(),child)
 assert i==len(data);return out
def lookup(root,path):
 node=root;parts=path.split('/')
 for i,part in enumerate(parts):
  mode,node=tree(node)[part]
  if i<len(parts)-1:assert mode=='40000'
 return mode,node
assert lookup(roots[r0['commit']],'crates/fabelgeist-numpy-storage/src')==lookup(roots[old],'crates/fabelgeist-numpy-storage/src')
current_crate=lookup(roots[r0['commit']],'crates/fabelgeist-numpy-storage')[1];old_crate=lookup(roots[old],'crates/fabelgeist-numpy-storage')[1];assert current_crate!=old_crate
x=tree(current_crate);y=tree(old_crate);assert {k for k in x.keys()|y.keys() if x.get(k)!=y.get(k)}=={'Cargo.toml'}
sources={};wanted={};rowmap={};archives={}
for reg in regs:
 for r in reg['rows']:
  name=r['source_file'];rowmap.setdefault(name,r);wanted.setdefault(name,set())
  for x,y in r['registered_ranges']:wanted[name].update(range(x,y+1))
  if name in sources:assert sha(sources[name])==r['sha256'];continue
  if r['kind']=='current Git source':
   data=(p/name).read_bytes();assert lookup(roots[r0['commit']],r['path'])==('100644',r['git_blob']) and gid('blob',data)==r['git_blob']
   if not r['path'].endswith('/Cargo.toml'):assert lookup(roots[old],r['path'])==('100644',r['git_blob'])
  else:
   key=(r['package'],r['version']);arc=a.archives_dir/(key[0]+'-'+key[1]+'.crate')
   if key not in archives:
    raw=arc.read_bytes();assert sha(raw)==r['archive_sha256'];archives[key]=arc
   with tarfile.open(arc,'r:gz') as t:data=t.extractfile(r['archive_member']).read()
   if (p/name).exists():assert (p/name).read_bytes()==data
   else:assert name==safe['source_file_omitted'] and sha(data)==safe['full_source_sha256']
  assert sha(data)==r['sha256'] and len(data)==r['bytes'] and len(data.splitlines())==r['line_count'];sources[name]=data
for oldrow in q['exact_source_identities_and_assessed_ranges']:
 sid=oldrow['source_id']
 if sid.startswith('native:rawzip@0.5.2:'):name='upstream/rawzip/src/archive.rs'
 else:name='first-party/'+sid.split(':')[2]
 assert name in sources and sha(sources[name])==oldrow['source_sha256']
for e in safe['excerpts']:
 lines=sources[safe['source_file_omitted']].splitlines(keepends=True);x,y=e['source_range'];data=b''.join(lines[x-1:y]);assert data==(p/e['path']).read_bytes() and len(data)==e['bytes'] and sha(data)==e['sha256']
actual={k:{} for k in wanted}
for d in sr['displays']:
 file=d['file'];reg=r2 if file.startswith('SOURCE-EXT2-') else r1 if file.startswith('SOURCE-EXT-') else r0;assert datetime.fromisoformat(d['observed_mtime_utc'])>=datetime.fromisoformat(reg['registered_utc']);out=[]
 for v in d['rows']:
  name=v['source_file'];lo,hi=v['range'];assert name in sources and any(rr['source_file']==name and any(x<=lo<=hi<=y for x,y in rr['registered_ranges']) for rr in reg['rows']);lines=sources[name].decode().splitlines();out.append(f'=== {name} {lo}..{hi} ===');out.extend(f'{i:4d} | {lines[i-1]}' for i in range(lo,hi+1))
  for i in range(lo,hi+1):actual[name][i]=actual[name].get(i,0)+1
 data=('\n'.join(out)+'\n').encode();assert data==(p/file).read_bytes() and len(data)==d['bytes'] and sha(data)==d['sha256']
for name,lines in wanted.items():assert set(actual[name])==lines
repeated={(name,i) for name,counts in actual.items() for i,n in counts.items() if n>1};assert repeated=={('upstream/rawzip/src/archive.rs',1200)}
for r in hreg['records']:
 data=(p/r['copy']).read_bytes();assert len(data)==r['bytes'] and sha(data)==r['sha256']
for v in hr['views']:
 data=(p/v['file']).read_bytes();assert len(data)==v['bytes'] and sha(data)==v['sha256'];reg=load('HISTORY-METADATA-REGISTRATION.json') if v['file'].startswith('HISTORY-DISPLAY-') else hreg;assert datetime.fromisoformat(v['observed_mtime_utc'])>=datetime.fromisoformat(reg['registered_utc'])
orig=(p/'QUEUE-ENTRY-ORIGINAL.json').read_bytes();rec=(p/'QUEUE-ENTRY-RECONSTRUCTION.json').read_bytes();assert orig==rec and sha(orig)==extract['records']['original']['entry_sha256']==extract['records']['reconstruction']['entry_sha256'] and q['contract_id']=='REVIEW-L11-NPZ-DECLARED-SIZE-CRC-ALLOCATION-POLICY' and not q['implementation_gate_complete'] and q['body_or_population_credit']==0
for file,lo,hi in [('HISTORY-DISPLAY-A.txt',1,75),('HISTORY-DISPLAY-B.txt',76,150),('HISTORY-DISPLAY-C.txt',151,225),('HISTORY-DISPLAY-D.txt',226,293)]:
 lines=orig.decode().splitlines();out=f'=== QUEUE-ENTRY-ORIGINAL.json {lo}..{hi} ===\n'+'\n'.join(f'{i:4d} | {lines[i-1]}' for i in range(lo,hi+1))+'\n';assert (p/file).read_text()==out
hist=p/'historical-originals';v=load('HISTORY-PROJECTION-A.json')['views']
for z in v:
 d=json.loads((hist/z['source']).read_bytes());assert d['revision']==z['revision'] and all(d[k]==z[k] for k in ['population_credit','implementation_gate_complete','native_runtime_validation'])
 for row in z['rows']:
  rr=d['rows'][row['row_index']];assert all(rr[k]==val for k,val in row['fields'].items())
v=load('HISTORY-PROJECTION-B.json')
for z in v['root_views']:
 d=json.loads((hist/z['source']).read_bytes());assert all(d[k]==val for k,val in z['fields'].items())
 for part,full in zip(z['dispositions'],d['finding_connections_and_scope_dispositions']):assert all(part[k]==full.get(k) for k in part)
d=json.loads((hist/'parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-004.json').read_bytes())['contract_updates'][7];assert all(d[k]==v for k,v in v['proposal_contract_7'].items())
child=tomllib.loads(sources['first-party/crates/fabelgeist-numpy-storage/Cargo.lock'].decode());man=tomllib.loads(sources['first-party/crates/fabelgeist-numpy-storage/Cargo.toml'].decode());names=['rawzip','flate2','miniz_oxide','crc32fast','zlib-rs'];assert [x for x in child['package'] if x['name'] in names]==meta['child_packages'] and {k:v for k,v in man['dependencies'].items() if k in ['rawzip','flate2']}==meta['child_direct_dependencies']
lock_packages={(x['name'],x['version']):x.get('checksum') for x in child['package']}
for reg in regs:
 for r in reg['rows']:
  if r['kind']!='current Git source':assert lock_packages[(r['package'],r['version'])]==r['archive_sha256']
if a.original_queue:
 data=a.original_queue.read_bytes();e=extract['records']['original'];assert sha(data)==e['queue_sha256'] and len(data)==e['queue_bytes'] and data[e['byte_start']:e['byte_end_exclusive']]==orig;print('original 059 full queue hash and exact q2316 slice verified')
else:print('original full queue not supplied; recorded exact slice only')
rootlock=a.root_lock or (a.publication_root/'lock-evidence/current-Cargo.lock' if a.publication_root else None)
if rootlock:
 raw=rootlock.read_bytes();assert sha(raw)==meta['root_lock_sha256'];d=tomllib.loads(raw.decode());assert [x for x in d['package'] if x['name'] in names]==meta['root_packages'];print('recorded root lock metadata verified separately')
else:print('root lock not supplied; root-lock metadata not independently replayed')
lreg=load('LEDGER-SEARCH-REGISTRATION.json');lr=load('LEDGER-SEARCH-RESULT.json');assert lr['registration_sha256']==sha((p/'LEDGER-SEARCH-REGISTRATION.json').read_bytes())
if a.ledger:
 raw=a.ledger.read_bytes();assert sha(raw)==lreg['source_sha256'];rows=json.loads(raw)['newly_assessed_ledger']['entries'];out=[]
 for i,r in enumerate(rows):
  if isinstance(r,dict) and any(t in json.dumps(r,ensure_ascii=True).lower() for t in lreg['terms']):out.append({'index':i,'id':r.get('id') or r.get('contract_id') or r.get('finding_id')})
 assert len(rows)==lr['entry_count'] and len(out)==lr['matches_total'] and out[:30]==lr['first_30'];print('time-bound ledger metadata search verified')
else:print('time-bound ledger metadata search not replayed')
if (p/'MANIFEST.json').exists():
 m=load('MANIFEST.json');fs={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert fs==set(m['files']) and all(not x.is_symlink() for x in p.rglob('*'))
 for name,v in m['files'].items():
  data=(p/name).read_bytes();assert len(data)==v['bytes'] and sha(data)==v['sha256']
print('commit/tree, child lock, three checksum-bound archives, source displays and historical records verified')
print('semantic assessment, actual feature selection, privacy beyond separate scans, and product behavior not verified')
