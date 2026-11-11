#!/usr/bin/env python3
"""Verify checkpoint recovery bytes, snapshot provenance and privacy only."""
import gzip,hashlib,io,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);out[n.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
def verify(body,m,where,depth=0):
 assert depth<=5 and sha(body)==m['published_sha256'] and len(body)==m['published_bytes'],where
 encoding=m['content_encoding'];assert encoding in ('identity','gzip','tar','gzip-tar');plain=gzip.decompress(body) if encoding.startswith('gzip') else body
 assert sha(plain)==m['published_uncompressed_sha256'] and len(plain)==m['published_uncompressed_bytes'] and m['changed']==(m['published_sha256']!=m['original_sha256'])
 if not m['changed']:assert len(body)==m['original_bytes'] and sha(plain)==m['original_uncompressed_sha256'] and len(plain)==m['original_uncompressed_bytes'] and m['total_local_path_redactions']==0
 else:assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions']>0
 if encoding.endswith('tar'):
  rows={x['path']:x for x in m['members']};assert len(rows)==len(m['members']) and m['direct_local_path_redactions']==0
  with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
   files=t.getmembers();assert len(files)==len(rows) and all((x.isfile() or x.isdir()) and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);assert set(x.name for x in files)==set(rows)
   for x in files:
    assert (x.uid,x.gid,x.uname,x.gname)==(0,0,'','') and not local.search(x.name.encode()) and not secret.search(x.name.encode());assert all(not local.search(str(v).encode()) and not secret.search(str(v).encode()) for v in x.pax_headers.values())
    if x.isdir():assert rows[x.name]['entry_type']=='directory' and not rows[x.name]['changed'] and rows[x.name]['total_local_path_redactions']==0
    else:verify(t.extractfile(x).read(),rows[x.name],where+'!'+x.name,depth+1)
  assert m['total_local_path_redactions']==sum(x['total_local_path_redactions'] for x in rows.values())
 else:assert not local.search(plain) and not secret.search(plain) and plain.count(b'<LOCAL_PATH_')>=m['direct_local_path_redactions'] and m['total_local_path_redactions']==m['direct_local_path_redactions'],where
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-9.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';p17='90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696';assert cp['working_revision']==rev and cp['historical_revision']==old
expected={'scene-tests':('/root/mhr_character',[2431,2432,2433]),'ahoy':('/root/hair_ops',[2435]),'quality-rules':('/root/rig_identity',[2354,2434]),'p17-orientation':('/root/mhr_character',[2436,2437]),'viewer-capture':('/root/hair_ops',[2438,2439]),'report-p17':('/root/rig_identity',[2440,2441]),'p17-capture':('/root/mhr_character',[2442,2443]),'viewer-limits':('/root/hair_ops',[2444,2445]),'p17-biomechanics':('/root/mhr_character',[2446,2447]),'p17-continuity':('/root/rig_identity',[2448,2449]),'p17-landing':('/root/hair_ops',[2450,2451]),'p17-metrics':('/root/mhr_character',[2452,2453]),'p17-metric-facing':('/root/rig_identity',[2454,2455]),'p17-height':('/root/mhr_character',[2456,2457]),'p17-peaks-pose-hair-ops':('/root/hair_ops',[2458,2459])}
transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 h,p=line.split('\t',1);tree[p]=h.split()[2]
by={};meta={}
for b in cp['bundles']:
 name=b['bundle_id'];assert name not in by and (b['agent'],b['scope_original_roles'])==expected[name];assert b['historical_source_revision']==(p17 if b['scope_original_roles'][0]>=2436 else old);raw=(root/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes'];members={x['path']:x for x in b['members']};assert len(members)==len(b['members'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  files=t.getmembers();assert len(files)==len(members) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files}
 assert set(data)==set(members)
 for path,body in data.items():verify(body,members[path],name+'/'+path)
 assert data[b['report_archive_member']]==(root/b['report_path']).read_bytes();by[name]=data;meta[name]=members
assert set(by)==set(expected) and sum(len(b['members']) for b in cp['bundles'])==1614 and sum(x['total_local_path_redactions'] for b in cp['bundles'] for x in b['members'])==40852
# Verify every indexed 059 record. Two public copies have documented path
# redactions; the exact originals were separately verified read-only.
redacted=[];count=0
for name,d in by.items():
 for index,key,pathkey,copykey in [('historical-manifest.json','records','original_transport_path','copy_path'),('history-manifest.json','original_059_records','original_path','preserved_bytes'),('historical-059-index.json','records','original_path','copy'),('original-attribution.json','original_059_records','original_path','preserved_bytes'),('related-059-receipts.json','records','original_path','preserved_bytes'),('historical-current-receipts.json','records','original_path','preserved_bytes')]:
  if index not in d:continue
  for x in json.loads(d[index])[key]:
   original=lookup[x[pathkey]];m=meta[name][x[copykey]];raw=d[x[copykey]];count+=1;assert x['sha256']==original['sha256']==m['original_sha256'] and x['bytes']==original['bytes']==m['original_bytes'] and x['git_blob']==original['git_blob']
   if m['changed']:redacted.append((name,x[pathkey],x[copykey],m['published_sha256']))
   else:assert sha(raw)==x['sha256'] and len(raw)==x['bytes'] and oid('blob',raw)==x['git_blob']
assert count==292
qual=load('DELEGATED-CHECKPOINT-9-QUALIFICATION.json');assert len(redacted)==len(qual['entries'])==2
for name,path,copy,pub in redacted:
 x=next(x for x in qual['entries'] if x['original_059_transport_path']==path);assert name=='scene-tests' and x['public_member']==copy and x['published_sha256']==pub and x['original_sha256']==lookup[path]['sha256'] and x['original_git_blob']==lookup[path]['git_blob'] and x['original_bytes']==lookup[path]['bytes'] and 'not byte-identical' in x['qualification']
# Assigned source bodies: registry identity is independent of personal reading.
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for name in ('scene-tests','p17-orientation','p17-capture','p17-biomechanics','p17-metrics','p17-height'):
  d=by[name];reg=json.loads(d['registration.json']);rows=reg['sources'];historic=[x for x in rows if x['kind'] in ('historical','report-p17')];assert sorted(x['original_role'] for x in historic)==sorted(expected[name][1])
  for x in rows:
   body=d[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
   if x['kind']=='current':assert x['revision']==rev and tree[x['path']]==x['git_blob_sha1']
   else:assert x['revision']==(old if name=='scene-tests' else p17) and body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
  for x in reg.get('absent',[]):assert x['revision']==rev and x['path'] not in tree
 d=by['ahoy'];rows=json.loads(d['registration.json'])['ordered_roles'];assert [x['original_index'] for x in rows]==[2435]
 for x in rows:
  for v in x['versions']:
   body=d[v['local_path']];assert sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']
   if v['revision']==rev:assert tree[x['path']]==v['git_blob']
   else:assert v['revision']==old and body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
 d=by['quality-rules'];rows=json.loads(d['registration.json'])['entries'];assert sorted(set(x['original_role'] for x in rows))==[2354,2434] and len(rows)==4
 for x in rows:
  current=x['revision']==rev;assert current or x['revision']==old;body=d['sources/'+('95e6/' if current else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
  if current:assert tree[x['path']]==x['git_blob']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
 for name in ('viewer-capture','viewer-limits','p17-landing','p17-peaks-pose-hair-ops'):
  d=by[name];roles=json.loads(d['registration.json'])['roles'];assert sorted(x['original_index'] for x in roles)==sorted(expected[name][1])
  for x in roles:
   for v in x['versions']:
    present=v.get('present',True)
    if not present:
     if v['revision']==rev:assert x['path'] not in tree
     continue
    body=d[v['local_path']];assert sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']
    if v['revision']==p17:assert body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
    elif v['revision']==rev:assert tree[x['path']]==v['git_blob']
    else:assert v['revision']==old
 for name in ('report-p17','p17-continuity','p17-metric-facing'):
  d=by[name];rows=json.loads(d['old-registration.json'])['entries'];assert sorted(x['original_role'] for x in rows)==sorted(expected[name][1])
  for x in rows:
   body=d['sources/90a4/'+x['path']];assert x['revision']==p17 and sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob'] and body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
  for x in json.loads(d['current-registration.json'])['entries']:
   assert x['revision']==rev
   if x['status']=='present':
    body=d['sources/95e6/'+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']==tree[x['path']]
   else:assert x['status']=='absent_at_current' and x['path'] not in tree
# Check raw Git path proofs, nonmembership and original 774 receipt identities.
for name in ('quality-rules','report-p17','p17-continuity','p17-metric-facing'):
 d=by[name];m=meta[name];p=json.loads(d['git-proofs.json']);objects={}
 for object_id,x in p['objects'].items():
  body=gzip.decompress(d[x['preserved_bytes']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
 for r,x in p['commits'].items():assert r==x['commit_object'] and objects[r].splitlines()[0]==b'tree '+x['root_tree'].encode()
 for x in p['paths']:
  cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]==x['path'].split('/')
  for s in x['path_chain']:assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['git_blob']
 for x in p.get('absent_paths',[]):
  cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]+[x['missing_segment']]+x['unvisited_segments']==x['path'].split('/')
  for s in x['path_chain']:assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['absent_from_tree'] and x['missing_segment'] not in entries(objects[cur])
 h=json.loads(d['history-manifest.json' if name=='quality-rules' else 'original-attribution.json'])
 for x in h['older_774_receipts']:
  proof=next(y for y in p['paths'] if y['revision']==x['transport_revision'] and y['path']==x['transport_path']);z=m[x['preserved_bytes']];assert x['sha256']==proof['sha256']==z['original_sha256'] and x['bytes']==proof['bytes']==z['original_bytes'] and x['git_blob']==proof['git_blob']
  if not z['changed']:assert oid('blob',d[x['preserved_bytes']])==x['git_blob']
# Alternate proof schema and original-vs-normalized 774 export pairs.
for name in ('scene-tests','p17-orientation','p17-capture','p17-biomechanics','p17-metrics','p17-height','p17-landing','p17-peaks-pose-hair-ops'):
 d=by[name];p=json.loads(d['git-proofs.json']);objects={}
 for object_id,x in p['objects'].items():
  body=gzip.decompress(d[x['archive']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
 for x in p['proofs']:
  assert objects[x['revision']].splitlines()[0]==b'tree '+x['commit_tree'].encode();cur=x['commit_tree'];assert [s['name'] for s in x['chain']]==x['path'].split('/')
  for s in x['chain']:assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['blob']==oid('blob',d[x['archive']])
 if 'git-absence-proofs.json' in d:
  for x in json.loads(d['git-absence-proofs.json'])['proofs']:
   assert objects[x['revision']].splitlines()[0]==b'tree '+x['commit_tree'].encode();cur=x['commit_tree'];assert [s['name'] for s in x['parent_chain']]+[x['missing_name']]+x.get('remaining_components',[])==x['path'].split('/')
   for s in x['parent_chain']:assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
   assert cur==x['parent_tree'] and x['missing_name'] not in entries(objects[cur])
 if 'older-774-index.json' in d:
  manifest=d['older-774-export-manifest.json'];assert sha(manifest)=='0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5';rows={x['file']:x for x in json.loads(manifest)['files']}
  for x in json.loads(d['older-774-index.json'])['records']:
   record=rows['records/'+x['path']];assert record['original_record_sha256']==x['original_sha256']==meta[name][x['original_copy']]['original_sha256'];assert record['published_export_sha256']==x['normalized_export_sha256']==sha(d[x['normalized_copy']]);assert x['distinct']==(x['original_sha256']!=x['normalized_export_sha256'])
# Explicitly retain partial reading and incident reconstruction qualification.
report=json.loads(by['quality-rules']['REPORT.json']);r=next(x for x in report['original_roles'] if x['original_role']==2434);assert r['current_unread_ranges'] and r['historical_unread_ranges'] and r['structural_parse_only_entire_file']
receipt=json.loads(by['p17-metrics']['read-receipt.json']);rows=[x for x in receipt['reads'] if x['role']==2452];assert len(rows)==2 and all(not x['full_body_displayed'] and x['unread_ranges'] for x in rows);assert next(x for x in rows if x['revision']==p17)['unread_ranges']==[[410,1184]] and next(x for x in rows if x['revision']==rev)['unread_ranges']==[[1284,2050]]
incident=load(cp['recovery_incident']);assert incident['original_packet']['unchanged_manifest_sha256']==meta['p17-metrics']['BUNDLE-MANIFEST.json']['original_sha256']
for x in incident['original_packet']['restored_files']:assert meta['p17-metrics'][x['name']]['original_sha256']==x['sha256']
assert 'reconstructed' in incident['recovery_qualification'] and incident['product_validation']=='not performed'
search=load('SEARCH-P16-PATCH-20261111.json');assert search['search']['matches']==0 and search['search']['files_hashed']==7156 and search['recorded_sha256']=='fc9002ab1135a438f68eede9217f67d22ef737d2c6938509e6e6660f2fc5b7f7'
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch9_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-9.json'
print('Verified 1614 top-level members, recursive archives, 290 exact and 2 qualified redacted 059 record references, source identities and Git proofs; no product execution.')
