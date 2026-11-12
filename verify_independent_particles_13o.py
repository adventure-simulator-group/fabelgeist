#!/usr/bin/env python3
"""Verify an independently attributed source review and exact archive."""
import hashlib,json,subprocess,sys,tarfile
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;packet=root/'delegated/independent-review-13o';archive=root/'delegated/independent-review-13o-archive.tar.gz';H=lambda b:hashlib.sha256(b).hexdigest()
assert H(archive.read_bytes())=='d1bc3dbdb196aa8fdaeb320fbebbf14e0b8d071872fc4a82e0bf3932c6d55612'
assert H((packet/'MANIFEST.json').read_bytes())=='061a851fc2d772788a04367101b4c32a3c1f1f2dbc4a6bd37cdf72c4eede2097'
assert H((packet/'REPORT.json').read_bytes())=='9761d614df0233802bbee641fa694937f683e197827eae430461c6ace5224836'
with tarfile.open(archive,'r:gz') as t:
 members=t.getmembers();seen=set();assert len(members)==95
 for member in members:
  p=PurePosixPath(member.name);assert member.isfile() and not p.is_absolute() and p.parts[0]=='bundle' and '..' not in p.parts
  rel=PurePosixPath(*p.parts[1:]);assert rel.as_posix() not in seen;seen.add(rel.as_posix());assert t.extractfile(member).read()==(packet/rel).read_bytes()
 assert seen=={x.relative_to(packet).as_posix() for x in packet.rglob('*') if x.is_file()}
subprocess.run([sys.executable,'-B',str(packet/'verify-evidence.py')],cwd=root,check=True)
scan=root/'delegated/independent-review-13o-scan';rb=(scan/'RECEIPT.json').read_bytes();assert H(rb)=='5e47ed5be1d4228fc33da88c6c98ca41af3f90863dc513097963d36236c00c24';rec=json.loads(rb);assert rec['archive_sha256']==H(archive.read_bytes())
for name,digest in rec['scans'].items():
 b=(scan/name).read_bytes();assert H(b)==digest;s=json.loads(b);assert s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
review=json.loads((packet/'REPORT.json').read_bytes());assert review['reviewer']=='/root/mhr_character' and review['reviewed_working_evidence']['assessment_sha256']==H((root/'ASSESSMENT-CENTRAL-PARTICLES-13O-95E.json').read_bytes())
add=json.loads((root/'ASSESSMENT-CENTRAL-PARTICLES-13O-REVIEW-ADDENDUM.json').read_bytes());assert add['review']=='delegated/independent-review-13o/REPORT.json' and add['reviewed_coordinator_assessment_sha256']==review['reviewed_working_evidence']['assessment_sha256']
print('Verified exact review archive, independently registered source identity receipts and additive connection; offline checks do not establish independent custody or product behavior.')
