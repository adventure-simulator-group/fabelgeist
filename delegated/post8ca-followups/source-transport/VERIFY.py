#!/usr/bin/env python3
"""Check every public review transport member without executing product code."""
from pathlib import Path,PurePosixPath
import hashlib,io,json,tarfile,sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();manifest=json.loads((p/'MANIFEST.json').read_bytes());mapping=json.loads((p/'ARCHIVE-MAP.json').read_bytes());assert manifest['schema']=='726-post8ca-public-transport-v1' and mapping['schema']=='726-post8ca-public-transport-map-v1' and len(mapping['collections'])==18
files=manifest['files'];assert set(files)=={'README.md','VERIFY.py','ARCHIVE-MAP.json'}|{x['archive'] for x in mapping['collections']} and {x.name for x in p.iterdir()}==set(files)|{'MANIFEST.json'}
for name,row in files.items():
 assert len(PurePosixPath(name).parts)==1;f=p/name;assert f.is_file() and not f.is_symlink();raw=f.read_bytes();assert (len(raw),h(raw))==(row['bytes'],row['sha256'])
labels=set();total=0
for row in mapping['collections']:
 label=row['label'];assert label not in labels and all(c.isalnum() or c=='-' for c in label);labels.add(label);raw=(p/row['archive']).read_bytes();assert (len(raw),h(raw))==(row['archive_bytes'],row['archive_sha256']);expected={x['path']:x for x in row['members']};assert len(expected)==len(row['members']) and label in expected and expected[label]['kind']=='directory';seen=set();captured_manifest=None
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
  for item in tar:
   name=item.name.rstrip('/');rel=PurePosixPath(name);assert not rel.is_absolute() and rel.parts and rel.parts[0]==label and '..' not in rel.parts and rel.as_posix()==name and name not in seen and name in expected and (item.isfile() or item.isdir());seen.add(name);want=expected[name];assert item.mode==int(want['mode'],8)
   if item.isdir():assert want['kind']=='directory'
   else:
    assert want['kind']=='file';data=tar.extractfile(item).read();assert (len(data),h(data))==(want['bytes'],want['sha256']);total+=1
    if name==label+'/MANIFEST.json':captured_manifest=h(data)
 assert seen==set(expected) and captured_manifest==row['original_manifest_sha256']
print(json.dumps({'collections':len(labels),'file_members':total,'qualification':'Checks all archived member bytes and modes, original public manifest identities and archive identities. It does not establish source semantics, execute packet verifiers, recover restricted originals, prove independent custody, close a lens or validate product behavior.'}))
