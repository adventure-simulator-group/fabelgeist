#!/usr/bin/env python3
"""Verify delegated audit followups and exact portable archive bytes."""
import hashlib,json,subprocess,sys,tarfile
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
sets=[
 ('independent-review-13r','302bf029c6ae7200a15553325b68ab14a223766a4015712883f9ee2ac56bd178','7b75b62b2a8bcd904a98e1d360f0b5533fff0949a053b22271d54bc49e137142',34,'independent-review-13r-coordinator-scan','748df6a93827685d86dc6067ac478f34cef6c03aa29bfcf913704c602a54bf69',['--publication',str(root)]),
 ('wrapped-scope-review','13420a33bd9db0af2a4a1d6373b57ae774fb13eb45340808ebd634b8cd38beea','4faf0d608202f8049920e3e8db7a3199b3140063fda6e102c1c994ffb8690a54',30,'wrapped-scope-review-scan','83c56f097b65cfa7ddcf5a186400b080fe3d2aedc313db981c52f25eeb5fa057',['--publication-root',str(root)]),
 ('capture-clock-q2300','1f06fee68757c65ba77a06d80203784d1085647c1f0ba1878961aaa7bf9392d4','3c32549fdae939775f9470c8c44d78858f898a3517bbf1a7e10a5b1e1b93d1c8',101,'capture-clock-q2300-scan','759f81524bb5512f2e9c48f2382b0ba61d8cdd360d44ea0be99a5eda26f744ad',[])]
for name,mh,ah,count,scanname,sh,args in sets:
 packet=root/'delegated'/name;mb=(packet/'MANIFEST.json').read_bytes();assert H(mb)==mh;rows=json.loads(mb)['files']
 if isinstance(rows,dict):rows=[{'path':k,**v} for k,v in rows.items()]
 assert len(rows)==count-1
 seen=set()
 for row in rows:
  p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen;seen.add(p.as_posix());b=(packet/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
 assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()}==seen|{'MANIFEST.json'}
 archive=root/'delegated'/(name+'-archive.tar.gz');assert H(archive.read_bytes())==ah
 with tarfile.open(archive,'r:gz') as t:
  members=t.getmembers();archived=set();assert len(members)==count
  for member in members:
   p=PurePosixPath(member.name);assert member.isfile() and not p.is_absolute() and p.parts[0]=='bundle' and '..' not in p.parts
   rel=PurePosixPath(*p.parts[1:]);assert rel.as_posix() not in archived;archived.add(rel.as_posix());assert t.extractfile(member).read()==(packet/rel).read_bytes()
  assert archived==seen|{'MANIFEST.json'}
 scan=root/'delegated'/scanname;recbytes=(scan/'RECEIPT.json').read_bytes();assert H(recbytes)==sh;rec=json.loads(recbytes);mapping=rec.get('scans',rec.get('files'))
 assert rec.get('archive_sha256',rec.get('reconstructed_archive_sha256'))==ah
 for fn,digest in mapping.items():
  data=(scan/fn).read_bytes();assert H(data)==digest
  if fn.endswith('.json'):
   result=json.loads(data)
   if 'forbidden_digest_total' in result:assert result['forbidden_digest_total']==0 and not result['other_candidate_counts'] and not result['exceptions']
   if 'counts' in result:assert not any(result['counts'].values())
 subprocess.run([sys.executable,'-B',str(packet/'verify-evidence.py'),*args],cwd=root,check=True)
 if name=='independent-review-13r':
  review=json.loads((packet/'REVIEW.json').read_bytes());assert review['reviewer']=='/root/verifier_scope' and review['reviewed_coordinator_draft_sha256']==H((root/'delegated/central-13r-fbx/ASSESSMENT-DRAFT.json').read_bytes())
  agent_scan=root/'delegated/independent-review-13r-agent-scan';am=(agent_scan/'MANIFEST.json').read_bytes();assert H(am)=='237c0a12b196ae61a814e7624d2760987f113fab188d87f5b41808d9c6ba1c3b'
  for row in json.loads(am)['files']:
   path=PurePosixPath(row['path']);assert not path.is_absolute() and '..' not in path.parts;b=(agent_scan/path).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
  sr=json.loads((agent_scan/'RECEIPT.json').read_bytes());assert sr['stage_manifest_sha256']==mh and H((agent_scan/'SCAN-RESULT.json').read_bytes())==sr['recursive_result_sha256'] and H((agent_scan/'PATTERN-RESULT.json').read_bytes())==sr['pattern_result_sha256']
  digest=json.loads((agent_scan/'SCAN-RESULT.json').read_bytes());patterns=json.loads((agent_scan/'PATTERN-RESULT.json').read_bytes());assert digest['forbidden_digest_total']==0 and not digest['other_candidate_counts'] and not digest['exceptions'] and not patterns['candidate_counts'] and not patterns['exceptions']
  add=json.loads((root/'ASSESSMENT-CENTRAL-FBX-13R-REVIEW-ADDENDUM.json').read_bytes());assert add['review']=='delegated/independent-review-13r/REVIEW.json'
 if name=='wrapped-scope-review':
  report=json.loads((packet/'REPORT.json').read_bytes());assert report['author']=='/root/wrapped_scope' and any(x['id']=='candidate-armor-design-error-bracer-prefix-95e' for x in report['findings'])
 if name=='capture-clock-q2300':
  report=json.loads((packet/'REPORT.json').read_bytes());assert report['author']=='/root/mhr_character' and report['historical_queue_pointer']=='baseline-059/queue.json#/entries/2300'
  replay=root/'delegated/capture-clock-q2300-replay';rec=json.loads((replay/'RECORD.json').read_bytes());log=(replay/'VERIFICATION.log').read_bytes();assert H(log)==rec['transcript_sha256'] and len(log)==rec['transcript_bytes'] and rec['packet_manifest_sha256']==mh
print('Verified three exact followup archives, registered evidence and bounded scans; optional private replay, current semantics and product behavior remain separately qualified.')
