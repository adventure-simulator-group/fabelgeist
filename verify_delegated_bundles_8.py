#!/usr/bin/env python3
"""Verify recursive recovery packages and attribution, not product behavior."""
import gzip,hashlib,io,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);out[n.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
def verify(body,m,location,depth=0):
 assert depth<=4 and sha(body)==m['published_sha256'] and len(body)==m['published_bytes'],location
 encoding=m['content_encoding'];assert encoding in ('identity','gzip','tar','gzip-tar')
 plain=gzip.decompress(body) if encoding.startswith('gzip') else body
 assert sha(plain)==m['published_uncompressed_sha256'] and len(plain)==m['published_uncompressed_bytes']
 assert m['changed']==(m['published_sha256']!=m['original_sha256'])
 if not m['changed']:
  assert len(body)==m['original_bytes'] and sha(plain)==m['original_uncompressed_sha256'] and len(plain)==m['original_uncompressed_bytes'] and m['total_local_path_redactions']==0
 else:assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions']>0
 if encoding.endswith('tar'):
  assert m['direct_local_path_redactions']==0 and 'members' in m
  rows={x['path']:x for x in m['members']};assert len(rows)==len(m['members'])
  with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
   files=t.getmembers();assert len(files)==len(rows) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files)
   assert set(x.name for x in files)==set(rows)
   for x in files:
    assert (x.uid,x.gid,x.uname,x.gname)==(0,0,'','')
    assert not local.search(x.name.encode()) and not secret.search(x.name.encode())
    assert all(not local.search(str(v).encode()) and not secret.search(str(v).encode()) for v in x.pax_headers.values())
    verify(t.extractfile(x).read(),rows[x.name],location+'!'+x.name,depth+1)
  assert m['total_local_path_redactions']==sum(x['total_local_path_redactions'] for x in rows.values())
 else:
  assert not local.search(plain) and not secret.search(plain),location
  assert plain.count(b'<LOCAL_PATH_')>=m['direct_local_path_redactions'] and m['total_local_path_redactions']==m['direct_local_path_redactions']
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-8.json');assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cp['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e'
expected={'health-template':('/root/mhr_character',[2399,2400,2401]),'skill-templates':('/root/hair_ops',[2402,2403]),'trade-templates':('/root/rig_identity',[2409,2410,2411,2412]),'web-chrome':('/root/mhr_character',[2404,2405,2406]),'settlement-templates':('/root/hair_ops',[2407,2408,2413]),'strategic-scripts':('/root/rig_identity',[2414,2419,2420,2421,2422]),'web-trade-scripts':('/root/mhr_character',[2417,2418]),'generation-js':('/root/hair_ops',[2423,2424,2425,2426,2427]),'scene':('/root/rig_identity',[2428,2429,2430])}
transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 h,p=line.split('\t',1);tree[p]=h.split()[2]
by={};meta={};nested_count=0
for b in cp['bundles']:
 name=b['bundle_id'];assert name not in by and (b['agent'],b['scope_original_roles'])==expected[name];raw=(root/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes'];members={x['path']:x for x in b['members']};assert len(members)==len(b['members'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  files=t.getmembers();assert len(files)==len(members) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files}
 assert set(data)==set(members)
 for path,body in data.items():
  verify(body,members[path],name+'/'+path);nested_count+=int(members[path]['content_encoding']=='gzip-tar')
 assert data[b['report_archive_member']]==(root/b['report_path']).read_bytes();by[name]=data;meta[name]=members
assert set(by)==set(expected) and sum(len(x['members']) for x in cp['bundles'])==1384 and nested_count==5
assert sum(x['total_local_path_redactions'] for b in cp['bundles'] for x in b['members'])==13918
# All 267 copied 059 records are byte-for-byte original transport entries.
history=[('health-template','historical-059-index.json','records','original_path','copy',25),('skill-templates','historical-manifest.json','records','original_transport_path','copy_path',37),('trade-templates','history-manifest.json','original_059_records','original_path','preserved_bytes',35),('web-chrome','historical-059-index.json','records','original_path','copy',25),('settlement-templates','historical-manifest.json','records','original_transport_path','copy_path',39),('strategic-scripts','history-manifest.json','original_059_records','original_path','preserved_bytes',48),('web-trade-scripts','historical-059-index.json','records','original_path','copy',21),('generation-js','historical-manifest.json','records','original_transport_path','copy_path',13),('scene','history-manifest.json','original_059_records','original_path','preserved_bytes',24)]
assert sum(x[-1] for x in history)==267
for name,index,key,pathkey,copykey,count in history:
 d=by[name];rows=json.loads(d[index])[key];assert len(rows)==count
 for x in rows:
  original=lookup[x[pathkey]];raw=d[x[copykey]];assert not meta[name][x[copykey]]['changed'];assert sha(raw)==x['sha256']==original['sha256'] and len(raw)==x['bytes']==original['bytes'] and oid('blob',raw)==x['git_blob']==original['git_blob']
# Assigned source bytes match the historical archive and the current Git tree.
# Archival identity, planned scope and personal display remain separate facts.
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for name in ('skill-templates','settlement-templates','generation-js'):
  d=by[name];rows=json.loads(d['registration.json'])['ordered_roles'];assert sorted(x['original_index'] for x in rows)==sorted(expected[name][1])
  for x in rows:
   assert len(x['versions'])==2
   for v in x['versions']:
    current=v['revision']==cp['working_revision'];assert current or v['revision']==cp['historical_revision'];body=d['source/'+('95e6-' if current else '6533-')+Path(x['path']).name];assert v['present'] and sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']
    if current:assert tree[x['path']]==v['git_blob']
    else:assert body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
  for x in json.loads(d['reference/manifest.json'])['records']:
   if x.get('copy') in meta[name]:assert x['sha256']==meta[name][x['copy']]['original_sha256'] and x['bytes']==meta[name][x['copy']]['original_bytes']
 for name in ('trade-templates','strategic-scripts','scene'):
  d=by[name];rows=json.loads(d['registration.json'])['entries'];assert sorted(set(x['original_role'] for x in rows))==sorted(expected[name][1]) and len(rows)==2*len(expected[name][1])
  for x in rows:
   current=x['revision']==cp['working_revision'];assert current or x['revision']==cp['historical_revision'];body=d['sources/'+('95e6/' if current else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
   if current:assert tree[x['path']]==x['git_blob']
   else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
 for name in ('health-template','web-chrome','web-trade-scripts'):
  d=by[name];rows=json.loads(d['registration.json'])['sources'];assert sorted(set(x['original_role'] for x in rows))==sorted(expected[name][1]) and len(rows)==2*len(expected[name][1])
  for x in rows:
   body=d[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
   if x['kind']=='current':assert x['revision']==cp['working_revision'] and tree[x['path']]==x['git_blob_sha1']
   else:assert x['kind']=='historical' and x['revision']==cp['historical_revision'] and body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
# Rig's raw Git objects prove both present paths and explicit absent tree entry.
for name in ('trade-templates','strategic-scripts','scene'):
 d=by[name];members=meta[name];p=json.loads(d['git-proofs.json']);objects={}
 for object_id,x in p['objects'].items():
  body=gzip.decompress(d[x['preserved_bytes']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
 for rev,x in p['commits'].items():assert rev==x['commit_object'] and objects[rev].splitlines()[0]==b'tree '+x['root_tree'].encode()
 for x in p['paths']:
  cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]==x['path'].split('/')
  for s in x['path_chain']:
   assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['git_blob']
 for x in p.get('absent_paths',[]):
  cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]+[x['missing_segment']]+x['unvisited_segments']==x['path'].split('/')
  for s in x['path_chain']:
   assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['absent_from_tree'] and x['missing_segment'] not in entries(objects[cur])
 h=json.loads(d['history-manifest.json'])
 for x in h['older_774_receipts']:
  proof=next(y for y in p['paths'] if y['revision']==x['transport_revision'] and y['path']==x['transport_path']);m=members[x['preserved_bytes']];assert x['sha256']==proof['sha256']==m['original_sha256'] and x['bytes']==proof['bytes']==m['original_bytes'] and x['git_blob']==proof['git_blob']
  if not m['changed']:assert oid('blob',d[x['preserved_bytes']])==x['git_blob']
 for x in h['prior_current_agent_records']:
  m=members[x['preserved_bytes']];assert x['previous_agent'] and x['sha256']==m['original_sha256'] and x['bytes']==m['original_bytes']
  if not m['changed']:assert sha(d[x['preserved_bytes']])==x['sha256']
# MHR raw Git proofs and exact-vs-normalized historical 774 copies.
for name in ('health-template','web-chrome','web-trade-scripts'):
 d=by[name];p=json.loads(d['git-proofs.json']);objects={}
 for object_id,x in p['objects'].items():
  body=gzip.decompress(d[x['archive']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
 for x in p['proofs']:
  assert objects[x['revision']].splitlines()[0]==b'tree '+x['commit_tree'].encode();cur=x['commit_tree'];assert [s['name'] for s in x['chain']]==x['path'].split('/')
  for s in x['chain']:
   assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
  assert cur==x['blob']==oid('blob',d[x['archive']])
 manifest=d['older-774-export-manifest.json'];assert sha(manifest)=='0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5';rows={x['file']:x for x in json.loads(manifest)['files']}
 for x in json.loads(d['older-774-index.json'])['records']:
  record=rows['records/'+x['path']];assert record['original_record_sha256']==x['original_sha256']==meta[name][x['original_copy']]['original_sha256'];assert record['published_export_sha256']==x['normalized_export_sha256']==sha(d[x['normalized_copy']]);assert x['distinct']==(x['original_sha256']!=x['normalized_export_sha256'])
assert len(cp['current_session_clarifications'])==3 and all(x['agent']=='/root/hair_ops' and '95e' in x['statement'] for x in cp['current_session_clarifications'])
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch8_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-8.json'
print('Verified 1384 top-level members, five recursive bundles, 267 exact 059 records, sources and Git proofs; no product execution.')
