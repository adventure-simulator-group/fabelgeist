"""Verify preserved manual bytes; never probe a product or filesystem scenario."""
import gzip,hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parent
m=json.loads((r/'MANIFEST.json').read_text())
for x in m['files']:
 p=Path(x['path'])
 assert not p.is_absolute() and '..' not in p.parts
 raw=(r/p).read_bytes()
 assert len(raw)==x['stored_bytes'] and hashlib.sha256(raw).hexdigest()==x['stored_sha256']
 data=gzip.decompress(raw)
 assert len(data)==x['uncompressed_bytes'] and hashlib.sha256(data).hexdigest()==x['uncompressed_sha256']
print('verified',len(m['files']),'Linux manual and copyright evidence members')
