#!/usr/bin/env python3
"""Check sixth delegated recovery packet and Git objects, not product behavior."""
import gzip,hashlib,io,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);out[n.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-delegate-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-6.json');assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cp['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e';transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 h,p=line.split('\t',1);tree[p]=h.split()[2]
by={};meta={}
for b in cp['bundles']:
 name=b['bundle_id'];assert name not in by;raw=(root/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes'];members={x['path']:x for x in b['members']};assert len(members)==len(b['members'])
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  files=t.getmembers();assert len(files)==len(members) and all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files}
 assert set(data)==set(members)
 for p,body in data.items():
  x=members[p];assert sha(body)==x['published_sha256'] and len(body)==x['published_bytes'];plain=gzip.decompress(body) if x['content_encoding']=='gzip' else body;assert sha(plain)==x['published_uncompressed_sha256'] and not local.search(plain) and not secret.search(plain)
  if x['redacted_nonessential_local_path_occurrences']:assert plain.count(b'<LOCAL_PATH_')==x['redacted_nonessential_local_path_occurrences'] and 'Reconstructed public copy' in x['qualification']
  else:assert sha(body)==x['original_sha256'] and len(body)==x['original_bytes'] and sha(plain)==x['original_uncompressed_sha256']
 assert data[b['report_archive_member']]==(root/b['report_path']).read_bytes();by[name]=data;meta[name]=members
assert set(by)=={'web-dialogue','web-map','web-routes','web-equipment'} and sum(len(x['members']) for x in cp['bundles'])==347
for name,index,key,pathkey,copykey,count in [('web-dialogue','historical-manifest.json','records','original_transport_path','copy_path',24),('web-map','history-manifest.json','original_059_records','original_path','preserved_bytes',20),('web-routes','historical-059-index.json','records','original_path','copy',29),('web-equipment','historical-manifest.json','records','original_transport_path','copy_path',29)]:
 d=by[name];rows=json.loads(d[index])[key];assert len(rows)==count
 for x in rows:
  original=lookup[x[pathkey]];raw=d[x[copykey]];assert sha(raw)==x['sha256']==original['sha256'] and len(raw)==x['bytes']==original['bytes'] and oid('blob',raw)==x['git_blob']==original['git_blob']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for name in ('web-dialogue','web-equipment'):
  d=by[name]
  for x in json.loads(d['registration.json'])['ordered_roles']:
   basename=Path(x['path']).name;body=d['source/'+('95e6-' if name=='web-equipment' else '')+basename];assert body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
   if name=='web-equipment':assert body==d['source/6533-'+basename]
   for v in x['versions']:assert v['present'] and sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']==tree[x['path']]
 d=by['web-map']
 for x in json.loads(d['registration.json'])['entries']:
  body=d['sources/'+('95e6/' if x['revision'].startswith('95e6') else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
  if x['revision'].startswith('95e6'):assert tree[x['path']]==x['git_blob']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
 d=by['web-routes']
 for x in json.loads(d['registration.json'])['sources']:
  body=d[x['archive']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
  if x['kind']=='current':assert tree[x['path']]==x['git_blob_sha1']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
# Map Git objects and all current/historical/older transport path chains.
d=by['web-map'];members=meta['web-map'];p=json.loads(d['git-proofs.json']);objects={}
for object_id,x in p['objects'].items():
 body=gzip.decompress(d[x['preserved_bytes']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
for rev,x in p['commits'].items():assert rev==x['commit_object'] and objects[rev].splitlines()[0]==b'tree '+x['root_tree'].encode()
for x in p['paths']:
 cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]==x['path'].split('/')
 for s in x['path_chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
 assert cur==x['git_blob']
for x in json.loads(d['history-manifest.json'])['older_774_receipts']:
 proof=next(y for y in p['paths'] if y['revision']==x['transport_revision'] and y['path']==x['transport_path']);m=members[x['preserved_bytes']];assert x['sha256']==proof['sha256']==m['original_sha256'] and x['bytes']==proof['bytes']==m['original_bytes'] and x['git_blob']==proof['git_blob']
 if not m['redacted_nonessential_local_path_occurrences']:assert oid('blob',d[x['preserved_bytes']])==x['git_blob']
# Routes use a different proof schema and gzip-compressed raw Git objects.
d=by['web-routes'];p=json.loads(d['git-proofs.json']);objects={}
for object_id,x in p['objects'].items():
 body=gzip.decompress(d[x['archive']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
for x in p['proofs']:
 assert objects[x['revision']].splitlines()[0]==b'tree '+x['commit_tree'].encode();cur=x['commit_tree'];assert [s['name'] for s in x['chain']]==x['path'].split('/')
 for s in x['chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
 assert cur==x['blob']==oid('blob',d[x['archive']])
# Different 774 manifest bytes are matched but not silently substituted.
manifest=d['older-774-export-manifest.json'];assert sha(manifest)=='0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5';rows={x['file']:x for x in json.loads(manifest)['files']}
for x in json.loads(d['older-774-index.json'])['records']:
 record=rows['records/'+x['path']];assert record['original_record_sha256']==x['original_sha256']==meta['web-routes'][x['original_copy']]['original_sha256'];assert record['published_export_sha256']==x['normalized_export_sha256']==sha(d[x['normalized_copy']]);assert x['distinct']==(x['original_sha256']!=x['normalized_export_sha256'])
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch6_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-6.json'
print('Verified 347 members, 102 original 059 records, source bodies and Git path proofs; no product execution.')
