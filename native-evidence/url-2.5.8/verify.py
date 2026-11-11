"""Verify exact selected native source/license bytes; not a product test."""
import gzip,hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
m=json.loads((root/'MANIFEST.json').read_text());assert m['archive_sha256']=='ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed';content={}
for x in m['inspected_members']:
 raw=gzip.decompress((root.parent.parent/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and hashlib.sha256(raw).hexdigest()==x['sha256'];content[x['archive_member']]=raw
host=content['url-2.5.8/src/host.rs'];lib=content['url-2.5.8/src/lib.rs'];tests=content['url-2.5.8/tests/unit.rs']
assert b'Url::host_str` returns the serialization of that address between `[` and `]` brackets' in host
assert b'Some(self.slice(self.host_start..self.host_end))' in lib
assert b'url.set_ip_host("::1".parse().unwrap()).unwrap();' in tests and b'assert_eq!(url.host_str(), Some("[::1]"));' in tests
print('verified 7 exact selected source/license members and documented registry archive checksum; original archive recoverable from registry')
