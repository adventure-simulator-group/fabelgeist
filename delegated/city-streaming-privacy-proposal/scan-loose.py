#!/usr/bin/env python3
"""Scan bounded public loose file bytes for named references, without following symlinks."""
from pathlib import Path
import argparse,datetime,hashlib,json,os
p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();r=a.root
patterns={'original_352_sha':'66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63','original_353_sha':'537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a','original_352_gzip_sha':'7dc8d1b8d59a3f1641cbd7efbb7d56788eb776b5f4d5d3935865e82e7397c358','original_353_gzip_sha':'007f948a973ac75bb1bb5d53fe9cd9105206b0e57b8e4ec58cf6be5f1ab29392','cloud_352_sha':'db3da4b2f0b6dc3d16aa448c5089b9b60ae62fd8400cb1eb72b956b6951c89c8','cloud_353_sha':'96f7f274a6b9d50a196a528eb1be9cad5d5d3ddfdc2aa870a898ddeb4401b415','original_path_352':'REGISTRATION-ADLER-352.json','original_path_353':'REGISTRATION-ADLER-353.json','city_directory':'recovered/city-streaming-059/','city_assessment':'ASSESSMENT-CITY-STREAMING-READINESS-95E.json','city_verifier':'verify_city_streaming_readiness_95e.py','city_assessment_sha':'d3d75ce6212526a047b08677495488f6fb8820ecbc9bdbef19f766a73e1a84ae','city_verifier_sha':'278e44689fd1310aadc62127958bcc32d555d871055d147d1c5fc0547bf15e9d'}; patterns.update({'cloud_352_gzip_sha': '82b9471f873416d514f7c9b5aed52f1afe5901a9a53ab2f0425c5f0b7afae112', 'cloud_353_gzip_sha': 'd4dfb06c8be60de5f776b06afa637a29aa4ff94a8ba441353aad96de38478b5c', 'original_352_decoded_blob': '4443fc6c4d0af983dfb8fb7016580eaa203e8811', 'original_353_decoded_blob': '7222745855fae84e10b3d3964d9c743aadbaeacd', 'original_352_gzip_blob': '99a458b3ee9a30cf2b493479b55ae6528895ce74', 'original_353_gzip_blob': 'a2b20159848f6f0bd262eec3844c682d085006ab', 'cloud_352_gzip_blob': 'f3fff9f90d84ed8b4c7461e94cc5d4790ba8c93f', 'cloud_353_gzip_blob': 'c69aafc738fc59db0aa40f8d37311635a5fa2d3a', 'cloud_352_decoded_blob': '64f6e223dd5d05c597d8c894cfd54153a807711e', 'cloud_353_decoded_blob': '04442178d26e386eaed2f4ea44599619973e5aa5'}); patterns={k:v.encode() for k,v in patterns.items()}
cap=64*1024*1024; paths=[]; symlinks=0
for base, dirs, files in os.walk(r,followlinks=False):
 dirs[:]=[d for d in dirs if d!='.git' and not (Path(base)/d).is_symlink()]
 for f in files:
  q=Path(base)/f
  if q.is_symlink():symlinks+=1;continue
  if q.is_file() and not q.name.endswith(('.gz','.tgz','.crate')):paths.append(q)
matches=[];unscanned=[];errors=[];changed=[];start=datetime.datetime.now(datetime.timezone.utc).isoformat()
for q in sorted(paths):
 rel=str(q.relative_to(r))
 try:
  before=q.stat()
  if before.st_size>cap:unscanned.append({'path':rel,'bytes':before.st_size,'reason':'stored bytes >64MiB cap'});continue
  b=q.read_bytes();after=q.stat()
  if (before.st_size,before.st_mtime_ns)!=(after.st_size,after.st_mtime_ns):changed.append(rel)
  keys=[k for k,v in patterns.items() if v in b]
  if keys:
   found={k:{'occurrences':sum(line.count(patterns[k]) for line in b.splitlines()),'lines':[i for i,line in enumerate(b.splitlines(),1) if patterns[k] in line]} for k in keys};matches.append({'path':rel,'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'matches':found})
 except OSError as e:errors.append({'path':rel,'error':type(e).__name__})
out={'scope':'all enumerated regular loose files outside .git with gzip/tgz/crate suffixes excluded (covered separately); no symlink traversal; byte patterns only','started_utc':start,'completed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'enumerated_files':len(paths),'symlink_files_skipped':symlinks,'limit_bytes':cap,'patterns':list(patterns),'matches':matches,'unscanned':unscanned,'changed_during_read':changed,'errors':errors,'qualification':'Literal references do not alone imply executable or live-byte dependency. Dynamic references, omissions, unrecognized compression, future and concurrent updates are not covered.'}
a.output.write_text(json.dumps(out,indent=2)+'\n');print('files',len(paths),'matches',len(matches),'unscanned',len(unscanned),'changed',len(changed),'errors',len(errors))
