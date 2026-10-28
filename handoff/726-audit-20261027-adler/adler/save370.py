from pathlib import Path
import bookkeeping as b
from record_l11 import record
parts=[]
expected=1
for seq in range(1,7):
 p=b.OUT/f'READ-L11-370-PARTIAL-{seq:03}.json';v=b.read(p)
 assert v['fresh_reads'][0][0]==expected
 expected=v['read_through']+1
 assert v['source_sha256']=='45961a0c74b57fb81e9429abc14d387a4e4a00ba18a771c17b0fd5e4ea09eaf4'
 parts.append(dict(receipt=str(p),sha256=b.sha(p),read_ranges=v['fresh_reads']))
 for old in v['partial_chain']:assert b.sha(Path(old['receipt']))==old['sha256']
assert expected==7094
record(370,369,[(2313,1173,[[1,7093]])],[dict(source_indices=[2313],ranges=[[1,7093]],roles='Independent lens11 frozen requested lockfile identity and dependency-name records only.',partial_receipts=parts,read_provenance='Cumulative first independent personally read source ranges, retained six exact continuous partial receipts; no reread claim. Each partial granted zero ordered credit. Full1..7093 now complete.',disposition='Generated Cargo lock v4 package/version/source/checksum and named dependency records read in full. Burn0.22pre2 Gitb1502de8, CubeCL0.11pre2 Git088dd705, CubeK0.3pre2 Git93a27826; WGPU/Naga30.0.1, anyhow1.0.104, reqwest0.12.28, thiserror1.0.69/2.0.20, serde1.0.229. Gitpliron26106513 andtracelLLVM0853480b pins included. Local MHR->FBX/rig/fs/numpy-storage dependencies recorded; native API bodies and enabled platform/backend/capability features NOT inferred from lock package presence. Multiple native/web/CPU/GPU backend records and dependency major versions do not themselves prove deprecated compatibility code. All20 owners/current/native error and shape admission remain pending. No dependency resolution/build/test/scanner/generator/runtime execution.')],[369],['370 accounts2313full7093 via six hash-linked cumulative first-read partial receipts, next2314MHRmanifest.22 mandatory all13 scoped extensions/six strictEOF qualifications/remaining future same-lens proofs retained. No other-lens verdict transfer.','Lock dependency identities only; actual MHR tensor/layout/topology and ndarray/native error bodies need source comparison, not inferred behavior from package names. Existing hair pending contracts unchanged.','All13 populations/current/all20/aliases/connected acceptance/dedup remain incomplete, gatefalse and accepteduniquechange0. No product repairs/builds/tests/lints/scanners/generators/runtime probes/CI/newtests/commits/publication/externalmessages. Goalactive.'],2314)
