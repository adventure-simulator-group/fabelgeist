"""Verify lock-bound crate archives and selected source bytes, never execute them."""
import gzip,hashlib,json,tarfile,tomllib
from pathlib import Path
r=Path(__file__).resolve().parent; root=r.parent.parent
m=json.loads((r/'MANIFEST.json').read_text())
lock=tomllib.loads(gzip.decompress((root/m['lockfile_source']).read_bytes()).decode())
packages={(x['name'],x['version']):x for x in lock['package']}
for x in m['crates']:
 raw=(r/x['path']).read_bytes()
 assert len(raw)==x['bytes'] and hashlib.sha256(raw).hexdigest()==x['sha256']==x['lockfile_checksum']==packages[(x['name'],x['version'])]['checksum']
 with tarfile.open(r/x['path'],'r:gz') as ar:
  for y in x['inspected_members']:
   data=ar.extractfile(y['path']).read()
   assert len(data)==y['bytes'] and hashlib.sha256(data).hexdigest()==y['sha256']
  assert all(f"{x['name']}-{x['version']}/LICENSE-{license}" in ar.getnames() for license in ('APACHE','MIT'))
print('verified',len(m['crates']),'lock-bound glTF archives and selected source members')
