#!/usr/bin/env python3
"""Bounded city registration reference and byte-identity scan; no extraction or writes to publication."""
from pathlib import Path
import argparse,datetime,gzip,hashlib,io,json,tarfile,sys
ap=argparse.ArgumentParser();ap.add_argument('--root',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
r=args.root;patterns={'original_352_sha': b'66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63', 'original_353_sha': b'537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a', 'original_352_gzip_sha': b'7dc8d1b8d59a3f1641cbd7efbb7d56788eb776b5f4d5d3935865e82e7397c358', 'original_353_gzip_sha': b'007f948a973ac75bb1bb5d53fe9cd9105206b0e57b8e4ec58cf6be5f1ab29392', 'cloud_352_sha': b'db3da4b2f0b6dc3d16aa448c5089b9b60ae62fd8400cb1eb72b956b6951c89c8', 'cloud_353_sha': b'96f7f274a6b9d50a196a528eb1be9cad5d5d3ddfdc2aa870a898ddeb4401b415', 'original_path_352': b'REGISTRATION-ADLER-352.json', 'original_path_353': b'REGISTRATION-ADLER-353.json', 'city_directory': b'recovered/city-streaming-059/', 'city_assessment': b'ASSESSMENT-CITY-STREAMING-READINESS-95E.json', 'city_verifier': b'verify_city_streaming_readiness_95e.py', 'city_assessment_sha': b'd3d75ce6212526a047b08677495488f6fb8820ecbc9bdbef19f766a73e1a84ae', 'city_verifier_sha': b'278e44689fd1310aadc62127958bcc32d555d871055d147d1c5fc0547bf15e9d'};patterns.update({'cloud_352_gzip_sha': b'82b9471f873416d514f7c9b5aed52f1afe5901a9a53ab2f0425c5f0b7afae112', 'cloud_353_gzip_sha': b'd4dfb06c8be60de5f776b06afa637a29aa4ff94a8ba441353aad96de38478b5c', 'original_352_decoded_blob': b'4443fc6c4d0af983dfb8fb7016580eaa203e8811', 'original_353_decoded_blob': b'7222745855fae84e10b3d3964d9c743aadbaeacd', 'original_352_gzip_blob': b'99a458b3ee9a30cf2b493479b55ae6528895ce74', 'original_353_gzip_blob': b'a2b20159848f6f0bd262eec3844c682d085006ab', 'cloud_352_gzip_blob': b'f3fff9f90d84ed8b4c7461e94cc5d4790ba8c93f', 'cloud_353_gzip_blob': b'c69aafc738fc59db0aa40f8d37311635a5fa2d3a', 'cloud_352_decoded_blob': b'64f6e223dd5d05c597d8c894cfd54153a807711e', 'cloud_353_decoded_blob': b'04442178d26e386eaed2f4ea44599619973e5aa5'});identities={'7dc8d1b8d59a3f1641cbd7efbb7d56788eb776b5f4d5d3935865e82e7397c358': 'stored_original_352', '007f948a973ac75bb1bb5d53fe9cd9105206b0e57b8e4ec58cf6be5f1ab29392': 'stored_original_353', '82b9471f873416d514f7c9b5aed52f1afe5901a9a53ab2f0425c5f0b7afae112': 'stored_cloud_352', 'd4dfb06c8be60de5f776b06afa637a29aa4ff94a8ba441353aad96de38478b5c': 'stored_cloud_353', '66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63': 'decoded_original_352', '537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a': 'decoded_original_353', 'db3da4b2f0b6dc3d16aa448c5089b9b60ae62fd8400cb1eb72b956b6951c89c8': 'decoded_cloud_352', '96f7f274a6b9d50a196a528eb1be9cad5d5d3ddfdc2aa870a898ddeb4401b415': 'decoded_cloud_353'}
sha=lambda b:hashlib.sha256(b).hexdigest();cap=64*1024*1024;limit=8;matches=[];unscanned=[];errors=[];archives=[];cache={};member_count=0;gzip_count=0;inner_tar_count=0

def scan(b):
 key=sha(b)
 if key in cache:return cache[key]
 found={}
 if key in identities:found["exact_"+identities[key]]={"occurrences":1,"lines":[]}
 if not any(v in b for v in patterns.values()):
  if len(cache)<50000:cache[key]=found
  return found
 for no,line in enumerate(b.splitlines(),1):
  for k,v in patterns.items():
   n=line.count(v)
   if n:
    x=found.setdefault(k,{'occurrences':0,'lines':[]});x['occurrences']+=n;x['lines'].append(no)
 if len(cache)<50000:cache[key]=found
 return found

def body(b,archive,chain,depth):
 global gzip_count,inner_tar_count
 found=scan(b)
 if found:matches.append({'archive':archive,'member_chain':chain,'representation':'stored/member bytes','bytes':len(b),'sha256':sha(b),'matches':found})
 if not b.startswith(b'\x1f\x8b'):return
 gzip_count+=1
 if depth>=limit:
  unscanned.append({'archive':archive,'member_chain':chain,'reason':'nested compression depth limit','stored_bytes':len(b),'stored_sha256':sha(b)});return
 try:
  with gzip.GzipFile(fileobj=io.BytesIO(b)) as g:unpacked=g.read(cap+1)
 except (OSError,EOFError) as e:
  errors.append({'archive':archive,'member_chain':chain,'reason':'gzip decode '+type(e).__name__});return
 if len(unpacked)>cap:
  unscanned.append({'archive':archive,'member_chain':chain,'reason':'decompressed member exceeds 64 MiB scan cap','stored_bytes':len(b),'stored_sha256':sha(b)});return
 # A tar payload must be parsed as a tar, not searched as opaque bytes only.
 if len(unpacked)>=512 and (unpacked[257:262] == b'ustar' or chain.endswith(('.tar.gz','.tgz'))):
  inner_tar_count+=1
  try:
   with tarfile.open(fileobj=io.BytesIO(unpacked),mode='r:') as t:
    for m in t:
     if not m.isfile():continue
     nested=m.name
     if m.size>cap:
      unscanned.append({'archive':archive,'member_chain':chain+'!'+nested,'reason':'tar member exceeds 64 MiB scan cap','bytes':m.size});continue
     body(t.extractfile(m).read(),archive,chain+'!'+nested,depth+1)
    stop=t.offset; tail=unpacked[stop:stop+512]
    if tail and any(tail):unscanned.append({'archive':archive,'member_chain':chain,'reason':'tar parser stopped before nonzero header; remaining entries unread','parser_offset':stop,'remaining_uncompressed_bytes':len(unpacked)-stop})
  except (tarfile.TarError,OSError,EOFError) as e: errors.append({'archive':archive,'member_chain':chain,'reason':'nested tar '+type(e).__name__})
 else:
  found=scan(unpacked)
  if found:matches.append({'archive':archive,'member_chain':chain,'representation':'gzip-uncompressed bytes','bytes':len(unpacked),'sha256':sha(unpacked),'matches':found})

start=datetime.datetime.now(datetime.timezone.utc).isoformat();paths=sorted((r/'delegated').rglob('*.tar.gz'))
for i,p in enumerate(paths,1):
 if p.is_symlink():unscanned.append({'archive':str(p.relative_to(r)),'reason':'symlink omitted'});continue
 rel=str(p.relative_to(r));before=p.stat()
 try:
  with p.open('rb') as f:h=hashlib.file_digest(f,'sha256').hexdigest()
  with tarfile.open(p,mode='r|gz') as t:
   for m in t:
    if not m.isfile():continue
    member_count+=1
    if m.size>cap:
     unscanned.append({'archive':rel,'member_chain':m.name,'reason':'outer member exceeds 64 MiB scan cap','bytes':m.size});continue
    raw=t.extractfile(m).read();body(raw,rel,m.name,0)
   stop=t.offset
  with gzip.open(p,'rb') as g:g.seek(stop);tail=g.read(512)
  if tail and any(tail):unscanned.append({'archive':rel,'reason':'outer tar parser stopped before nonzero header; remaining entries unread','parser_offset':stop})
  after=p.stat();changed=(before.st_mtime_ns,before.st_size)!=(after.st_mtime_ns,after.st_size)
  archives.append({'path':rel,'sha256':h,'bytes':before.st_size,'changed_while_scanned':changed})
 except Exception as e:errors.append({'archive':rel,'reason':'outer archive '+type(e).__name__+': '+str(e)[:160]})
 if i%25==0:print('scanned',i,'of',len(paths),'archives;',member_count,'outer members;',len(matches),'matching representations;',len(unscanned),'bounded omissions',flush=True)
out={'schema':'726-baseline059-delegated-archive-reference-inventory-v1','started_utc':start,'completed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All enumerated delegated *.tar.gz at scan start, reading files in tar order with bounded recursion and no extraction. Exact byte pattern scan of member bytes and gzip-uncompressed representations; tar metadata and filenames are not source-read credit.','limits':{'max_member_or_decompressed_bytes':cap,'max_nested_compression_depth':limit,'symlinks_followed':False},'archive_count_enumerated':len(paths),'archive_records':archives,'outer_regular_member_count':member_count,'gzip_members_seen':gzip_count,'nested_tar_streams_seen':inner_tar_count,'matching_representations':matches,'bounded_unscanned':unscanned,'errors':errors,'patterns':list(patterns),'qualification':'Literal matches are references, not proof that an archived script is active or that a hash names the current top-level file. Listed omissions, archive errors, non-.tar.gz formats and future/concurrently modified paths remain open.'}
args.output.write_text(json.dumps(out,indent=2)+'\n');print('done archives',len(archives),'matches',len(matches),'omissions',len(unscanned),'errors',len(errors),flush=True)
