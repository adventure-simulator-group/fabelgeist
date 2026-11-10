"""Verify a source-only dependency buffer contract trace and its provenance."""
import gzip,hashlib,json,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-REPLICON-BUFFER-95E.json'; doc=json.loads((root/name).read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); out[p]=oid
 return out
current=tree('main-advance-95e/current-tree.tsv'); previous=tree('main-advance-95e/previous-tree.tsv'); sources={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root); raw=gzip.decompress(p.read_bytes()); blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert s['revision']==rev and blob==s['git_blob']==current[s['path']]==previous[s['path']] and len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); sources[s['path']]=raw.decode()
app=sources['crates/adventuresim-tactical-netcode/src/replication.rs']; assert 'message: &mut Bytes' in app and 'let json: Vec<u8> = postcard_utils::from_buf(message)?' in app and 'nightly-2026-07-13' in sources['rust-toolchain.toml']
lock=gzip.decompress((root/'main-advance-95e/basis-wire-sources/01.source.gz').read_bytes()); assert hashlib.sha1(b'blob '+str(len(lock)).encode()+b'\0'+lock).hexdigest()==current['Cargo.lock']; packages=tomllib.loads(lock.decode())['package']; bodies={}
for package in ['bevy-replicon-0.41.1-archive','bytes-1.12.1','postcard-1.1.3','serde_core-1.0.229']:
 d=root/'native-evidence'/package; m=json.loads((d/'MANIFEST.json').read_text()); a=(d/m['archive']).read_bytes(); assert len(a)==m['bytes'] and hashlib.sha256(a).hexdigest()==m['sha256']; pk=[x for x in packages if x['name']==m['package'] and x['version']==m['version']]; assert len(pk)==1 and pk[0]['checksum']==m['sha256']
 with tarfile.open(d/m['archive']) as t:
  for x in m['inspected_members']:
   raw=t.extractfile(x['path']).read(); assert len(raw)==x['bytes'] and hashlib.sha256(raw).hexdigest()==x['sha256']; bodies[x['path']]=raw.decode()
rep=bodies['bevy_replicon-0.41.1/src/postcard_utils.rs']; trait=bodies['bytes-1.12.1/src/buf/buf_impl.rs']; chain=bodies['bytes-1.12.1/src/buf/chain.rs']; bytes_impl=bodies['bytes-1.12.1/src/bytes.rs']; postcard=bodies['postcard-1.1.3/src/de/deserializer.rs']; flavor=bodies['postcard-1.1.3/src/de/flavors.rs']; serde=bodies['serde_core-1.0.229/src/de/impls.rs']
for x in ('pub fn from_buf<\'de, T: Deserialize<\'de>, B: Buf>', 'if self.buf.remaining() < ct', 'slice::from_raw_parts(self.buf.chunk().as_ptr(), ct)', 'self.buf.advance(ct)'): assert x in rep
for x in ('This value is greater than or equal to the length of the slice returned', 'can* return a shorter slice'): assert x in trait
for x in ('self.a.remaining().saturating_add(self.b.remaining())','self.a.chunk()','self.b.chunk()'): assert x in chain
assert 'impl Buf for Bytes' in bytes_impl and 'self.len()' in bytes_impl[bytes_impl.index('impl Buf for Bytes'):] and 'self.as_slice()' in bytes_impl[bytes_impl.index('impl Buf for Bytes'):]
assert 'fn deserialize_f32<V>' in postcard and 'self.flavor.try_take_n_temp(4)?' in postcard and 'self.try_take_n(ct)' in flavor and 'f32, deserialize_f32' in serde and 'deserializer.$deserialize(PrimitiveVisitor)' in serde
native=json.loads((root/'native-evidence/rust-nightly-2026-07-13/MANIFEST.json').read_text()); r=next(x for x in native['files'] if x['path']=='library/core/src/slice/raw.rs.gz'); raw=gzip.decompress((root/'native-evidence/rust-nightly-2026-07-13'/r['path']).read_bytes()); assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256'] and b'entire memory range of this slice must be contained within a single allocation' in raw
h=doc['historical_source']; old=json.loads((root/h['ledger']).read_text())['discovery_findings_and_dependencies']; assert all(not any(term.lower() in (k+' '+json.dumps(v)).lower() for k,v in old.items()) for term in h['search_terms']); c=doc['candidates'][0]; b=json.loads((root/'BASELINE-95E-20261110.json').read_text())['newly_assessed_ledger']['entries']; p=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert sum(x['id']==c['id'] and x['record']==name for x in b)==1 and p['new_current_ledger_candidates']==len(b) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified lock-bound dependency source contracts and concrete Bytes scope; no decoder executed')
