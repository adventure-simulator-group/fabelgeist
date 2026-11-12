#!/usr/bin/env python3
"""Verify independent incidence review archive and bounded evidence."""
import hashlib,json,subprocess,sys,tarfile
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;packet=root/'delegated/independent-review-13p';archive=root/'delegated/independent-review-13p-archive.tar.gz';H=lambda b:hashlib.sha256(b).hexdigest()
assert H(archive.read_bytes())=='5d64af95d18aa257dd7b8d3628d973b774e4f28e5f4501cceab512ed11ba3568'
assert H((packet/'MANIFEST.json').read_bytes())=='38f33be88ff59b1469e1480b0d3b512553227ca7dc65d9501a262a04f477fd68'
assert H((packet/'REPORT.json').read_bytes())=='c30f3887e907221c75b780905ab44d2963e6376395d75825ecfb9f8f0cf4e661'
with tarfile.open(archive,'r:gz') as t:
 members=t.getmembers();seen=set();assert len(members)==95
 for member in members:
  p=PurePosixPath(member.name);assert member.isfile() and not p.is_absolute() and p.parts[0]=='bundle' and '..' not in p.parts
  rel=PurePosixPath(*p.parts[1:]);assert rel.as_posix() not in seen;seen.add(rel.as_posix());assert t.extractfile(member).read()==(packet/rel).read_bytes()
 assert seen=={x.relative_to(packet).as_posix() for x in packet.rglob('*') if x.is_file()}
subprocess.run([sys.executable,'-B',str(packet/'verify-evidence.py')],cwd=root,check=True)
scan=root/'delegated/independent-review-13p-scan';rb=(scan/'RECEIPT.json').read_bytes();assert H(rb)=='6d5f008d58e1e942f79181f3d4199e2806bd2081d7231970ec39c3fa6306facf';rec=json.loads(rb);assert rec['archive_sha256']==H(archive.read_bytes())
for name,digest in rec['scans'].items():
 b=(scan/name).read_bytes();assert H(b)==digest;s=json.loads(b);assert s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
review=json.loads((packet/'REPORT.json').read_bytes());assert review['reviewer']=='/root/mhr_character' and review['coordinator_packet']['manifest_sha256']==H((root/'delegated/central-13p-incidence/MANIFEST.json').read_bytes())
add=json.loads((root/'ASSESSMENT-CENTRAL-INCIDENCE-13P-REVIEW-ADDENDUM.json').read_bytes());assert add['review']=='delegated/independent-review-13p/REPORT.json' and add['reviewed_coordinator_manifest_sha256']==review['coordinator_packet']['manifest_sha256']
assert H((packet/'CAPTURE-IDENTITY-CORRECTION.json').read_bytes())=='b99cee2953876128edc276580c24a62a0fb1c2cbba69539a9a704d452d1468c1'
print('Verified exact incidence review archive and source evidence; historical discrepancies, private capture replay and product behavior remain separately qualified.')
