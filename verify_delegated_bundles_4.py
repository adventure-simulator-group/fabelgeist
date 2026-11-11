#!/usr/bin/env python3
"""Check fourth delegated recovery packet and Git objects, not product behavior."""
import gzip,hashlib,io,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,name=b[i:end].split(b' ',1);out[name.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-delegate-[^"\'\s]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-4.json');assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cp['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e'
transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
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
  if x['redacted_nonessential_local_path_occurrences']:
   assert plain.count(b'<LOCAL_PATH_')==x['redacted_nonessential_local_path_occurrences'] and 'Reconstructed public copy' in x['qualification']
  else:assert sha(body)==x['original_sha256'] and len(body)==x['original_bytes'] and sha(plain)==x['original_uncompressed_sha256']
 assert data[b['report_archive_member']]==(root/b['report_path']).read_bytes();by[name]=data;meta[name]=members
assert set(by)=={'shell-solver','strategic-web','xpbd-dynamics','xpbd-solver'} and sum(len(x['members']) for x in cp['bundles'])==419
for name,index,key,pathkey,copykey,count in [('shell-solver','historical-059-index.json','records','path','copy',31),('strategic-web','historical-manifest.json','records','original_transport_path','copy_path',30),('xpbd-dynamics','history-manifest.json','original_059_records','original_path','preserved_bytes',45),('xpbd-solver','historical-059-index.json','records','path','copy',33)]:
 d=by[name];rows=json.loads(d[index])[key];assert len(rows)==count
 for x in rows:
  original=lookup[x[pathkey]];raw=d[x[copykey]];assert sha(raw)==x['sha256']==original['sha256'] and len(raw)==x['bytes']==original['bytes'] and oid('blob',raw)==x['git_blob']==original['git_blob']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for name in ('shell-solver','xpbd-solver'):
  d=by[name]
  for x in json.loads(d['registration.json'])['sources']:
   body=d[x['source_file']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
   if x['kind']=='historical':assert body==ar.extractfile(work[x['original_role_index']]['historical_sha256']).read()
   else:assert tree[x['path']]==x['git_blob_sha1']
 d=by['strategic-web']
 for x in json.loads(d['registration.json'])['ordered_roles']:
  body=d['source/'+Path(x['path']).name]
  assert body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
  for v in x['versions']:assert v['present'] and sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']==tree[x['path']]
 d=by['xpbd-dynamics']
 for x in json.loads(d['registration.json'])['entries']:
  body=d['sources/'+('95e6/' if x['revision'].startswith('95e6') else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
  if x['revision'].startswith('95e6'):assert tree[x['path']]==x['git_blob']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
# Uncompressed commit/tree objects and all independently recorded path chains.
for name in ('shell-solver','xpbd-solver'):
 d=by[name]
 for p in json.loads(d['git-proofs.json'])['proofs']:
  commit=d['git-objects/'+p['revision']+'.commit'];assert oid('commit',commit)==p['revision'] and commit.splitlines()[0]==b'tree '+p['commit_tree'].encode();cur=p['commit_tree'];assert [x['entry'] for x in p['membership']]==p['path'].split('/')
  for step in p['membership']:
   raw=d['git-objects/'+cur+'.tree'];assert step['parent_tree']==cur and oid('tree',raw)==cur and entries(raw)[step['entry']]==(step['mode'].lstrip('0'),step['object']);cur=step['object']
  assert cur==p['blob_sha1']==oid('blob',d[p['source_file']])
# XPBD packet contains complete source and older transport Git path proofs.
d=by['xpbd-dynamics'];members=meta['xpbd-dynamics'];p=json.loads(d['git-proofs.json']);objects={}
for object_id,x in p['objects'].items():
 body=gzip.decompress(d[x['preserved_bytes']]);assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(x['type'],body)==object_id;objects[object_id]=body
for rev,x in p['commits'].items():assert rev==x['commit_object'] and objects[rev].splitlines()[0]==b'tree '+x['root_tree'].encode()
for x in p['paths']:
 cur=p['commits'][x['revision']]['root_tree'];assert [s['name'] for s in x['path_chain']]==x['path'].split('/')
 for s in x['path_chain']:
  assert s['tree']==cur and entries(objects[cur])[s['name']]==(s['mode'].lstrip('0'),s['object']);cur=s['object']
 assert cur==x['git_blob']
for x in json.loads(d['history-manifest.json'])['older_774_receipts']:
 proof=next(y for y in p['paths'] if y['revision']==x['transport_revision'] and y['path']==x['transport_path']);m=members[x['preserved_bytes']]
 assert x['sha256']==proof['sha256']==m['original_sha256'] and x['bytes']==proof['bytes']==m['original_bytes'] and x['git_blob']==proof['git_blob']
 if not m['redacted_nonessential_local_path_occurrences']:assert oid('blob',d[x['preserved_bytes']])==x['git_blob']
qualification=load('DELEGATED-CHECKPOINT-4-QUALIFICATION.json')
recovered=qualification['recovered_original_registration_records']
assert len(recovered)==2 and all(x['local_older_original_status']=='absent-at-checked-local-path' for x in recovered)
for x in recovered:
 assert x in json.loads(d['history-manifest.json'])['older_774_receipts']
 assert not members[x['preserved_bytes']]['redacted_nonessential_local_path_occurrences']
 assert sha(d[x['preserved_bytes']])==x['sha256'] and oid('blob',d[x['preserved_bytes']])==x['git_blob']
assert 'not proof of a personal read' in qualification['qualification']
report=json.loads(d['REPORT.json'])
for x in report['supporting_reads']:
 body=d[x['source']];assert sha(body)==x['sha256'] and oid('blob',body)==x['git_blob']==tree[x['path']]
old_support=report['historical_support_read'];body=d[old_support['source']];proof=next(x for x in p['paths'] if x['revision']==old_support['revision'] and x['path']==old_support['path']);assert sha(body)==proof['sha256'] and oid('blob',body)==proof['git_blob']
# Preserve original transfer-manifest bytes and distinguish normalized copies.
for name in ('shell-solver','xpbd-solver'):
 d=by[name];metadata=meta[name];index=json.loads(d['older-774/INDEX.json']);manifest=d[index['export_manifest_copy']];assert sha(manifest)==index['export_manifest_sha256']=='0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5';rows={x['file']:x for x in json.loads(manifest)['files']}
 for x in index['records']:
  assert x['manifest_entry']==rows[x['manifest_entry']['file']] and metadata[x['original_copy']]['original_sha256']==x['original_sha256']==x['manifest_entry']['original_record_sha256']
  if x['export_differs']:assert sha(d[x['normalized_copy']])==x['normalized_sha256']==x['manifest_entry']['published_export_sha256']
 for pair in load('DELEGATED-CHECKPOINT-3-QUALIFICATION.json')['shell_older_variant_pairs']:
  assert metadata[pair['shell_local_original_copy']]['original_sha256']==pair['historical_family_sha256']
  assert sha(d[pair['shell_path_normalized_copy']])==pair['adler_original_sha256']
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch4_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-4.json'
print('Verified 419 delegated members, 139 original 059 records, source bodies and older Git path proofs; no product execution.')
