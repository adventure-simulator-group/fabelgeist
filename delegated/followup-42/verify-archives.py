#!/usr/bin/env python3
"""Verify complete source-evidence archives and their public manifests."""
import hashlib,io,json,tarfile
from pathlib import Path,PurePosixPath
p=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest();d=json.loads((p/'CATALOG.json').read_bytes());assert d['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and d['product_validation']=='not performed'
assert {x['id'] for x in d['packets']}=={'hair-surface-grid','hair-collider','shell-mesh'}
roles=[n for x in d['packets'] for n in x['original_roles']];assert len(roles)==len(set(roles))==5 and set(roles)==set(range(2643,2648));total=0
for x in d['packets']:
 b=(p/x['archive']).read_bytes();assert (H(b),len(b))==(x['archive_sha256'],x['archive_bytes'])
 with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as t:
  members=t.getmembers();names=[m.name for m in members];assert len(names)==len(set(names))
  for m in members:
   q=PurePosixPath(m.name);assert m.isfile() and not q.is_absolute() and '..' not in q.parts and str(q)==m.name;assert (m.uid,m.gid,m.uname,m.gname,m.mtime,m.mode)==(0,0,'','',0,0o644)
  data={m.name:t.extractfile(m).read() for m in members};total+=len(data)
 raw=data[x['manifest']];assert H(raw)==x['manifest_sha256'];files=json.loads(raw)['files'];rows={v['path']:(v['sha256'],v['bytes']) for v in files} if isinstance(files,list) else {k:(v['sha256'],v['bytes']) for k,v in files.items()};assert len(rows)==len(files) and set(data)==set(rows)|{x['manifest']}
 for name,(sha,size) in rows.items():assert (H(data[name]),len(data[name]))==(sha,size),(x['id'],name)
for x in d['same_machine_receipts']:
 b=(p/x['path']).read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes'])
assert H((p/d['temporary_custody']).read_bytes())==d['temporary_custody_sha256'];incident=json.loads((p/d['temporary_custody']).read_bytes());assert 'unknown' in incident['outcome'] and 'read only' in incident['custody']
m=json.loads((p/'STAGE-MANIFEST.json').read_bytes());actual={x.relative_to(p).as_posix() for x in p.rglob('*') if x.is_file()}-{'STAGE-MANIFEST.json'};assert actual=={x['path'] for x in m['files']}
for x in m['files']:
 b=(p/x['path']).read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes'])
print('Verified three public archives,',total,'members, five distinct role identities and the temporary custody qualification; semantic replay, private originals and earlier history remain separate. No product ran.')
