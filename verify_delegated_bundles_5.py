#!/usr/bin/env python3
"""Check fifth delegated recovery packet, native source bytes and Git objects."""
import gzip,hashlib,io,json,re,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};i=0
 while i<len(b):
  end=b.index(b'\0',i);mode,n=b[i:end].split(b' ',1);out[n.decode()]=(mode.decode().lstrip('0'),b[end+1:end+21].hex());i=end+21
 assert i==len(b);return out
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-delegate-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-5.json');assert cp['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and cp['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e';transport=json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in transport['entries']};work=load('COVERAGE-L01-ROLES.json')['roles'];tree={}
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
assert set(by)=={'web-data','crate-metadata','mhr-metadata'} and sum(len(x['members']) for x in cp['bundles'])==463
redacted=[]
for name,index,key,pathkey,copykey,count in [('web-data','historical-manifest.json','records','original_transport_path','copy_path',24),('crate-metadata','history-manifest.json','original_059_records','original_path','preserved_bytes',87),('mhr-metadata','historical-059-index.json','records','path','copy',73)]:
 d=by[name];rows=json.loads(d[index])[key];assert len(rows)==count
 for x in rows:
  original=lookup[x[pathkey]];member=meta[name][x[copykey]];raw=d[x[copykey]];assert member['original_sha256']==x['sha256']==original['sha256'] and member['original_bytes']==x['bytes']==original['bytes'] and x['git_blob']==original['git_blob']
  if member['redacted_nonessential_local_path_occurrences']:redacted.append((name,x[pathkey],x[copykey]))
  else:assert sha(raw)==x['sha256'] and len(raw)==x['bytes'] and oid('blob',raw)==x['git_blob']
qual=load('DELEGATED-CHECKPOINT-5-QUALIFICATION.json');assert redacted==[('crate-metadata',qual['original_059_transport_path'],qual['public_member'])];original=lookup[qual['original_059_transport_path']];assert qual['original_sha256']==original['sha256'] and qual['original_git_blob']==original['git_blob'] and qual['original_bytes']==original['bytes'] and meta['crate-metadata'][qual['public_member']]['published_sha256']==qual['published_sha256'];assert 'not byte-identical' in qual['qualification']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 d=by['web-data']
 for x in json.loads(d['registration.json'])['ordered_roles']:
  body=d['source/'+Path(x['path']).name];assert body==ar.extractfile(work[x['original_index']]['historical_sha256']).read()
  for v in x['versions']:assert v['present'] and sha(body)==v['sha256'] and len(body)==v['bytes'] and oid('blob',body)==v['git_blob']==tree[x['path']]
 d=by['crate-metadata']
 for x in json.loads(d['registration.json'])['entries']:
  body=d['sources/'+('95e6/' if x['revision'].startswith('95e6') else '6533/')+x['path']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob']
  if x['revision'].startswith('95e6'):assert tree[x['path']]==x['git_blob']
  else:assert body==ar.extractfile(work[x['original_role']]['historical_sha256']).read()
 d=by['mhr-metadata']
 for x in json.loads(d['registration.json'])['sources']:
  body=d[x['source_file']];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid('blob',body)==x['git_blob_sha1']
  if x['kind']=='historical':assert body==ar.extractfile(work[x['original_role_index']]['historical_sha256']).read()
  else:assert tree[x['path']]==x['git_blob_sha1']
# MHR packet includes exact raw commits and source-path chains, including e275.
d=by['mhr-metadata']
for p in json.loads(d['git-proofs.json'])['proofs']:
 commit=d['git-objects/'+p['revision']+'.commit'];assert oid('commit',commit)==p['revision'] and commit.splitlines()[0]==b'tree '+p['commit_tree'].encode();cur=p['commit_tree'];assert [x['entry'] for x in p['membership']]==p['path'].split('/')
 for step in p['membership']:
  raw=d['git-objects/'+cur+'.tree'];assert step['parent_tree']==cur and oid('tree',raw)==cur and entries(raw)[step['entry']]==(step['mode'].lstrip('0'),step['object']);cur=step['object']
 assert cur==p['blob_sha1']==oid('blob',d[p['source_file']])
# Archived complete lock bytes are not a full personal read.
coverage=json.loads(d['lock-read-coverage.json'])['sources'];assert len(coverage)==2
receipt=json.loads(d['read-receipt.json'])
for x in coverage:
 assert sum(hi-lo+1 for lo,hi in x['personally_displayed_union'])+x['unread_lines']==x['lines']
 assert sum(hi-lo+1 for lo,hi in x['unread_ranges'])==x['unread_lines']
 partition=sorted(x['personally_displayed_union']+x['unread_ranges']);cursor=1
 for lo,hi in partition:assert lo==cursor and hi>=lo;cursor=hi+1
 assert cursor==x['lines']+1
 actual=[r for r in receipt['actual_source_displays']+receipt['additional_lock_displays'] if r['revision']==x['revision'] and r['path']=='crates/fabelgeist-mhr/Cargo.lock']
 merged=[]
 for lo,hi in sorted(interval for r in actual for interval in r['ranges']):
  if merged and lo<=merged[-1][1]+1:merged[-1][1]=max(merged[-1][1],hi)
  else:merged.append([lo,hi])
 assert merged==x['personally_displayed_union'] and all(r['sha256']==x['sha256'] for r in actual)
 if x['kind']=='current':assert x['lines']==7102 and x['unread_lines']==6904
 else:assert x['kind']=='historical' and x['lines']==7093 and x['unread_lines']==6898
# Crate packet includes Git objects/paths for three source revisions and f82.
d=by['crate-metadata'];members=meta['crate-metadata'];p=json.loads(d['git-proofs.json']);objects={}
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
# The original rawzip archive and five member bodies are matched to the child
# lock and original transport. No native source is executed or credited as read.
prefix='historical-059/parallel-l11-20261029/native-childlock-rawzip-192-20261030/'
archive=d[prefix+'rawzip-0.5.2.crate'];lock=d['support/crates/fabelgeist-numpy-storage/Cargo.lock'];assert oid('blob',lock)==tree['crates/fabelgeist-numpy-storage/Cargo.lock'];pkg=next(x for x in tomllib.loads(lock.decode())['package'] if x['name']=='rawzip' and x['version']=='0.5.2');assert pkg['checksum']==sha(archive)=='ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c'
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
 files=[x for x in t.getmembers() if x.isfile()];assert all(not t.extractfile(x).read(2)==b'\x1f\x8b' for x in files)
 for n in ('locator.rs','archive.rs','errors.rs','headers.rs','extra_fields.rs'):
  path='rawzip-0.5.2/src/'+n;body=t.extractfile(path).read();assert body==d[prefix+path]
# Earlier transfer manifest and path-normalized copies stay distinct.
d=by['mhr-metadata'];metadata=meta['mhr-metadata'];index=json.loads(d['older-774/INDEX.json']);manifest=d[index['export_manifest_copy']];assert sha(manifest)==index['export_manifest_sha256']=='0c848f3be21cf9013e3556d819564f01ee6d05136dff2dfe26c761bbe894ddd5';rows={x['file']:x for x in json.loads(manifest)['files']}
for x in index['records']:
 assert x['manifest_entry']==rows[x['manifest_entry']['file']] and metadata[x['original_copy']]['original_sha256']==x['original_sha256']==x['manifest_entry']['original_record_sha256']
 if x['export_differs']:assert sha(d[x['normalized_copy']])==x['normalized_sha256']==x['manifest_entry']['published_export_sha256']
assert load('CONTINUATION-PROGRESS-95E.json')['delegated_batch5_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-5.json'
print('Verified 463 members, 183 exact original 059 records, one qualified redacted record, Git proofs and rawzip bytes; no product execution.')
