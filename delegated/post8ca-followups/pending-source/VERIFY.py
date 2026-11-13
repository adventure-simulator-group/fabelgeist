#!/usr/bin/env python3
"""Verify every member of the later, unadopted public evidence transports."""
from pathlib import Path,PurePosixPath
import hashlib,io,json,tarfile,sys

if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());mapping=json.loads((p/'ARCHIVE-MAP.json').read_bytes());assert m['schema']=='726-post8ca-pending-transport-v1' and mapping['schema']=='726-post8ca-pending-transport-map-v1' and len(mapping['collections'])==8
files=m['files'];assert set(files)=={'README.md','VERIFY.py','ARCHIVE-MAP.json'}|{x['archive'] for x in mapping['collections']} and {x.name for x in p.iterdir()}==set(files)|{'MANIFEST.json'}
for name,row in files.items():
 assert len(PurePosixPath(name).parts)==1;f=p/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
labels=set();total=0
for row in mapping['collections']:
 label=row['label'];assert label not in labels and all(c.isalnum() or c=='-' for c in label);labels.add(label);b=(p/row['archive']).read_bytes();assert (len(b),h(b))==(row['archive_bytes'],row['archive_sha256']);expected={x['path']:x for x in row['members']};assert len(expected)==len(row['members']) and label in expected;seen=set();manifest=None
 with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as tar:
  for item in tar:
   name=item.name.rstrip('/');rel=PurePosixPath(name);assert not rel.is_absolute() and rel.parts and rel.parts[0]==label and '..' not in rel.parts and rel.as_posix()==name and name not in seen and name in expected and (item.isfile() or item.isdir());seen.add(name);want=expected[name];assert item.mode==int(want['mode'],8)
   if item.isdir():assert want['kind']=='directory'
   else:
    assert want['kind']=='file';data=tar.extractfile(item).read();assert (len(data),h(data))==(want['bytes'],want['sha256']);total+=1
    if name==label+'/MANIFEST.json':manifest=h(data)
 assert seen==set(expected) and manifest==row['original_manifest_sha256']
print(json.dumps({'collections':len(labels),'file_members':total,'qualification':'Checks archived public bytes and modes only; pending reports are not adopted, checkpoint3821 evidence, independent custody, source-semantic proof, audit closure or product validation.'}))
