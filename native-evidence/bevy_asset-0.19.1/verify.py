"""Verify retained pinned registry members; no native or product execution."""
import gzip,hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
m=json.loads((root/'MANIFEST.json').read_text());assert m['archive_sha256']=='1624eddbf2293441b2158506aba75e523432b898d9da26eb6ed3e20af4c7fd38';content={}
for x in m['inspected_members']:
 raw=gzip.decompress((root.parent.parent/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and hashlib.sha256(raw).hexdigest()==x['sha256'];content[x['archive_member']]=raw
server=content['bevy_asset-0.19.1/src/server/mod.rs'];info=content['bevy_asset-0.19.1/src/server/info.rs']
assert b'InternalAssetEvent::Failed { index, path, error }' in server and b'infos.process_asset_fail(index, error.clone())' in server
assert b'pub fn get_load_state(&self' in server and b'pub fn is_loaded_with_dependencies(&self' in server
assert b'info.load_state = LoadState::Failed(error.clone());' in info
print('Verified five exact selected source/license members and pinned registry archive checksum; no product execution.')
